use crate::services::connection_service::ConnectionCommand;
use derive_more::Display;
use std::ops::Deref;

use anyhow::{bail, Error, Result};
use chrono::{DateTime, Local};
use dioxus::hooks::use_coroutine_handle;
use dioxus::prelude::{spawn, Coroutine, GlobalSignal, Signal};
use ethersync_shared::keypair::Keypair;
use ethersync_shared::secret_address::SecretAddress;
use ethersync_shared::wormhole::get_secret_address_from_wormhole;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use iroh::endpoint::Incoming;
use iroh::{Endpoint, NodeId, SecretKey};

const ALPN: &[u8] = b"/ethersync/0";

#[derive(Clone, PartialEq)]
pub struct EthersyncNodeInfo {
    pub node_id: NodeId,
    pub my_passphrase: String,
    pub secret_key: String,
}

pub static NODE_INFO: GlobalSignal<Option<EthersyncNodeInfo>> = Signal::global(|| None);

pub enum NodeEvent {
    Error {
        date_time: DateTime<Local>,
        error: Error,
    },
    Spawned {
        date_time: DateTime<Local>,
    },
}

impl Display for NodeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeEvent::Error { date_time, error } => write!(f, "{date_time}: node error {error}"),
            NodeEvent::Spawned { date_time } => write!(f, "{date_time}: node spawned"),
        }
    }
}

pub static NODE_EVENTS: GlobalSignal<Vec<NodeEvent>> = Signal::global(Vec::new);

fn handle_error(error: Error) {
    NODE_EVENTS.write().push(NodeEvent::Error {
        date_time: Local::now(),
        error,
    });
}

fn generate_random_secret_key() -> SecretKey {
    SecretKey::generate(rand::thread_rng())
}

pub enum NodeCommand {
    ConnectByAddress { secret_address: Box<SecretAddress> },
    ConnectByJoinCode { join_code: String },
}

async fn create_endpoint(secret_key: SecretKey) -> Result<Endpoint> {
    Endpoint::builder()
        .secret_key(secret_key)
        .alpns(vec![ALPN.to_vec()])
        .discovery_n0()
        .bind()
        .await
}

async fn handle_incoming_connection(
    my_passphrase: SecretKey,
    incoming: Incoming,
    connection_service: Coroutine<ConnectionCommand>,
) -> Result<()> {
    let connection = incoming.await?;
    let (send, mut receive) = connection.accept_bi().await?;

    let mut received_passphrase = [0; 32];
    receive.read_exact(&mut received_passphrase).await?;

    // Guard against timing attacks.
    if !constant_time_eq::constant_time_eq(&received_passphrase, &my_passphrase.to_bytes()) {
        bail!("Peer provided incorrect passphrase.");
    }

    connection_service.send(ConnectionCommand::NewConnection {
        connection,
        receive,
        send,
    });

    Ok(())
}

async fn accept_incoming_connections(
    endpoint: Endpoint,
    my_passphrase: SecretKey,
    connection_service: Coroutine<ConnectionCommand>,
) {
    spawn(async move {
        loop {
            match endpoint.accept().await {
                None => break,
                Some(incoming) => {
                    if let Err(error) = handle_incoming_connection(
                        my_passphrase.clone(),
                        incoming,
                        connection_service,
                    )
                    .await
                    {
                        handle_error(error)
                    }
                }
            }
        }
    });
}

pub async fn connect(
    endpoint: Endpoint,
    secret_address: &SecretAddress,
    connection_service: Coroutine<ConnectionCommand>,
) -> Result<()> {
    let connection = endpoint.connect(secret_address.node_id, ALPN).await?;

    let (mut send, receive) = connection.open_bi().await?;

    send.write_all(&secret_address.passphrase.to_bytes())
        .await?;

    connection_service.send(ConnectionCommand::NewConnection {
        connection,
        receive,
        send,
    });

    Ok(())
}

async fn handle_node_command(
    endpoint: Endpoint,
    command: NodeCommand,
    connection_service: Coroutine<ConnectionCommand>,
) -> Result<()> {
    match command {
        NodeCommand::ConnectByAddress { secret_address } => {
            connect(endpoint.clone(), secret_address.deref(), connection_service).await
        }
        NodeCommand::ConnectByJoinCode { join_code } => {
            let secret_address = get_secret_address_from_wormhole(&join_code).await?;
            connect(endpoint.clone(), &secret_address, connection_service).await
        }
    }
}

pub async fn start_node_service(mut commands_rx: UnboundedReceiver<NodeCommand>) {
    let connection_service = use_coroutine_handle::<ConnectionCommand>();

    // TODO: store passphrase and allow changing it
    let keypair = Keypair {
        secret_key: generate_random_secret_key(),
        passphrase: generate_random_secret_key(),
    };

    match create_endpoint(keypair.secret_key.clone()).await {
        Ok(endpoint) => {
            *NODE_INFO.write() = Some(EthersyncNodeInfo {
                node_id: endpoint.node_id(),
                my_passphrase: keypair.passphrase.to_string(),
                secret_key: keypair.secret_key.to_string(),
            });
            NODE_EVENTS.write().push(NodeEvent::Spawned {
                date_time: Local::now(),
            });

            accept_incoming_connections(
                endpoint.clone(),
                keypair.passphrase.clone(),
                connection_service,
            )
            .await;

            while let Some(command) = commands_rx.next().await {
                if let Err(error) =
                    handle_node_command(endpoint.clone(), command, connection_service).await
                {
                    handle_error(error)
                }
            }
        }
        Err(error) => handle_error(error),
    }
}
