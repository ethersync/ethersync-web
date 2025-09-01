use anyhow::Result;
use automerge::sync::Message as AutomergeSyncMessage;
use dioxus::hooks::UnboundedReceiver;
use dioxus::prelude::spawn;
use futures::channel::mpsc::UnboundedSender;
use futures::{SinkExt, StreamExt};
use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::NodeId;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

pub enum EthersyncConnectionAction {
    AddConnection {
        connection: Connection,
        receive: RecvStream,
        send: SendStream,
    },
    SendMessage {
        message: AutomergeSyncMessage,
    },
}

pub enum EthersyncConnectionEvent {
    Connected {
        remote_node_id: NodeId,
        outgoing_message_tx: Sender<AutomergeSyncMessage>,
    },
    MessageReceived {
        remote_node_id: NodeId,
        message: AutomergeSyncMessage,
    },
    MessageSent {
        message: AutomergeSyncMessage,
    },
}

pub async fn receive_message(receive: &mut RecvStream) -> Result<AutomergeSyncMessage> {
    let mut message_len_buf = [0; 4];
    receive.read_exact(&mut message_len_buf).await?;
    let message_len = u32::from_be_bytes(message_len_buf);

    let mut message_buf = vec![0; message_len as usize];
    receive.read_exact(&mut message_buf).await?;

    Ok(AutomergeSyncMessage::decode(&message_buf)?)
}

fn start_receiving_messages(
    remote_node_id: NodeId,
    mut receive: RecvStream,
    mut event_tx: UnboundedSender<EthersyncConnectionEvent>,
) {
    spawn(async move {
        while let Ok(message) = receive_message(&mut receive).await {
            event_tx
                .send(EthersyncConnectionEvent::MessageReceived {
                    remote_node_id,
                    message,
                })
                .await
                .expect("failed to send event!");
        }
    });
}

pub async fn send_message(send: &mut SendStream, message: AutomergeSyncMessage) -> Result<()> {
    let message_buf = message.encode();
    let message_len = u32::try_from(message_buf.len()).expect("Failed to convert message length!");
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
            send_message(&mut send, message)
                .await
                .expect("failed to send message!");
        }
    });
}

pub async fn create_connection_handler(
    mut action_rx: UnboundedReceiver<EthersyncConnectionAction>,
    mut event_tx: UnboundedSender<EthersyncConnectionEvent>,
) {
    let (outgoing_message_tx, _) = broadcast::channel(16);

    while let Some(action) = action_rx.next().await {
        match action {
            EthersyncConnectionAction::AddConnection {
                connection,
                receive,
                send,
            } => {
                let remote_node_id = connection.remote_node_id().expect("no remote node id!");
                start_receiving_messages(remote_node_id, receive, event_tx.clone());
                start_sending_messages(send, outgoing_message_tx.subscribe());
                event_tx
                    .send(EthersyncConnectionEvent::Connected {
                        remote_node_id,
                        outgoing_message_tx: outgoing_message_tx.clone(),
                    })
                    .await
                    .expect("failed to send event!");
            }
            EthersyncConnectionAction::SendMessage { message } => {
                outgoing_message_tx
                    .send(message.clone())
                    .expect("failed to send!");
                event_tx
                    .send(EthersyncConnectionEvent::MessageSent { message })
                    .await
                    .expect("failed to send event!");
            }
        }
    }
}
