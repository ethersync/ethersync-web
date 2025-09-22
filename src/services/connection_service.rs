use crate::services::automerge_service::{AutomergeCommand, FormattedAutomergeMessage};
use crate::services::node_service::NODE_INFO;
use anyhow::{anyhow, Context, Error, Result};
use automerge::sync::Message as AutomergeSyncMessage;
use derive_more::{Deref, Display};
use dioxus::hooks::UnboundedReceiver;
use dioxus::prelude::{
    spawn, use_coroutine_handle, Coroutine, GlobalSignal, ReadableOptionExt, Signal,
};
use futures::StreamExt;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::NodeId;
use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

pub static AUTOMERGE_MESSAGES: GlobalSignal<Vec<FormattedAutomergeMessage>> =
    Signal::global(Vec::new);
pub static CONNECTED_PEERS: GlobalSignal<Vec<NodeId>> = Signal::global(Vec::new);
pub static CONNECTION_ERRORS: GlobalSignal<Vec<Error>> = Signal::global(Vec::new);

pub enum ConnectionCommand {
    NewConnection {
        connection: Connection,
        receive: RecvStream,
        send: SendStream,
    },
    SendMessage {
        message: AutomergeSyncMessage,
    },
}

pub type CursorId = String;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Paths like these are relative to the shared directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, Deref, Display)]
#[display("'{}'", self.0.display())]
#[must_use]
pub struct RelativePath(PathBuf);

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct CursorState {
    pub name: Option<String>,
    pub file_path: RelativePath,
    pub ranges: Vec<Range>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct EphemeralMessage {
    pub cursor_id: CursorId,
    pub sequence_number: usize,
    pub cursor_state: CursorState,
}

#[derive(Deserialize, Serialize)]
/// The `PeerMessage` is used for peer to peer data exchange.
pub enum PeerMessage {
    /// The Sync message contains the changes to the CRDT
    Sync(Vec<u8>),
    /// The Ephemeral message currently is used for cursor messages, but can later be used for
    /// other things that should not be persisted.
    Ephemeral(EphemeralMessage),
}

async fn receive_peer_message(receive: &mut RecvStream) -> Result<PeerMessage> {
    let mut message_len_buf = [0; 4];
    receive.read_exact(&mut message_len_buf).await?;
    let message_len = u32::from_be_bytes(message_len_buf);

    let mut message_buf = vec![0; message_len as usize];
    receive.read_exact(&mut message_buf).await?;

    from_bytes(&message_buf).context("Failed to convert bytes to PeerMessage")
}

async fn receive_message(receive: &mut RecvStream) -> Result<Option<AutomergeSyncMessage>> {
    match receive_peer_message(receive).await? {
        PeerMessage::Sync(message_buf) => Ok(Some(AutomergeSyncMessage::decode(&message_buf)?)),
        PeerMessage::Ephemeral(_message_buf) => {
            // TODO: implement the ephemerality
            Ok(None)
        }
    }
}

fn start_receiving_messages(
    remote_node_id: NodeId,
    mut receive: RecvStream,
    automerge_service: Coroutine<AutomergeCommand>,
) {
    spawn(async move {
        while let Ok(maybe_message) = receive_message(&mut receive).await {
            if let Some(message) = maybe_message {
                match FormattedAutomergeMessage::new("received", remote_node_id, &message) {
                    Ok(formatted_message) => {
                        automerge_service.send(AutomergeCommand::ApplyMessage { message });
                        AUTOMERGE_MESSAGES.write().push(formatted_message);
                    }
                    Err(error) => {
                        CONNECTION_ERRORS.write().push(error);
                    }
                }
            }
        }

        CONNECTED_PEERS.write().retain(|&n| n != remote_node_id);
    });
}

pub async fn send_message(send: &mut SendStream, message: AutomergeSyncMessage) -> Result<()> {
    let peer_message = PeerMessage::Sync(message.encode());
    let message_buf = to_allocvec(&peer_message)?;
    let message_len = u32::try_from(message_buf.len())?;
    send.write_all(&message_len.to_be_bytes()).await?;

    send.write_all(&message_buf).await?;

    Ok(())
}

fn start_sending_messages(
    mut send: SendStream,
    mut outgoing_message_rx: Receiver<AutomergeSyncMessage>,
) {
    spawn(async move {
        while let Ok(message) = outgoing_message_rx.recv().await {
            if let Err(error) = send_message(&mut send, message).await {
                CONNECTION_ERRORS.write().push(error);
            }
        }
    });
}

async fn handle_connection_command(
    command: ConnectionCommand,
    outgoing_message_tx: &Sender<AutomergeSyncMessage>,
    automerge_service: Coroutine<AutomergeCommand>,
) -> Result<()> {
    match command {
        ConnectionCommand::NewConnection {
            connection,
            receive,
            send,
        } => {
            let remote_node_id = connection.remote_node_id()?;
            start_receiving_messages(remote_node_id, receive, automerge_service);
            start_sending_messages(send, outgoing_message_tx.subscribe());

            CONNECTED_PEERS.write().push(remote_node_id);
            automerge_service.send(AutomergeCommand::StartSync);
        }
        ConnectionCommand::SendMessage { message } => {
            let node_id = NODE_INFO
                .as_ref()
                .map(|n| Ok(n.node_id))
                .unwrap_or(Err(anyhow!("missing node ID!")))?;

            match FormattedAutomergeMessage::new("sent", node_id, &message) {
                Ok(formatted_message) => {
                    outgoing_message_tx.send(message.clone())?;
                    AUTOMERGE_MESSAGES.write().push(formatted_message);
                }
                Err(error) => {
                    CONNECTION_ERRORS.write().push(error);
                }
            }
        }
    }
    Ok(())
}

pub async fn start_connection_service(mut commands_rx: UnboundedReceiver<ConnectionCommand>) {
    let automerge_service = use_coroutine_handle::<AutomergeCommand>();

    let (outgoing_message_tx, _) = broadcast::channel(16);

    while let Some(command) = commands_rx.next().await {
        if let Err(error) =
            handle_connection_command(command, &outgoing_message_tx, automerge_service).await
        {
            CONNECTION_ERRORS.write().push(error);
        }
    }
}
