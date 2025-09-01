use crate::shared::secret_address::SecretAddress;
use anyhow::{anyhow, Result};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use iroh::endpoint::{RecvStream, SendStream};
use iroh::{Endpoint, SecretKey};

const ALPN: &[u8] = b"/ethersync/0";

#[derive(Clone, PartialEq)]
pub struct EthersyncNodeInfo {
    pub node_id: String,
    pub my_passphrase: String,
    pub secret_key: String,
}

pub struct EthersyncNode {
    pub endpoint: Endpoint,
    pub my_passphrase: SecretKey,
    pub secret_key: SecretKey,
}

fn generate_random_secret_key() -> SecretKey {
    SecretKey::generate(rand::thread_rng())
}

impl EthersyncNode {
    pub fn node_info(&self) -> EthersyncNodeInfo {
        EthersyncNodeInfo {
            node_id: self.endpoint.node_id().to_string(),
            my_passphrase: self.my_passphrase.to_string(),
            secret_key: self.secret_key.to_string(),
        }
    }

    pub async fn spawn() -> Self {
        // TODO: store passphrase and allow changing it
        let my_passphrase = generate_random_secret_key();

        let secret_key = generate_random_secret_key();
        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .alpns(vec![ALPN.to_vec()])
            .discovery_n0()
            .bind()
            .await
            .expect("Could not bind endpoint!");
        Self {
            endpoint,
            my_passphrase,
            secret_key,
        }
    }

    pub async fn connect(
        &self,
        secret_address: SecretAddress,
    ) -> Result<(iroh::endpoint::Connection, RecvStream, SendStream)> {
        let connection = self
            .endpoint
            .connect(secret_address.peer_node_id, ALPN)
            .await?;

        let (mut send, receive) = connection.open_bi().await?;

        send.write_all(&secret_address.peer_passphrase.to_bytes())
            .await?;

        Ok((connection, receive, send))
    }

    pub async fn accept(&self) -> Result<(iroh::endpoint::Connection, RecvStream, SendStream)> {
        let incoming = self.endpoint.accept().await;
        if incoming.is_none() {
            return Err(anyhow!("endpoint closed!"));
        }

        let connection = incoming.unwrap().await?;
        let (send, mut receive) = connection.accept_bi().await?;

        let mut received_passphrase = [0; 32];
        receive.read_exact(&mut received_passphrase).await?;

        // Guard against timing attacks.
        if !constant_time_eq::constant_time_eq(&received_passphrase, &self.my_passphrase.to_bytes())
        {
            return Err(anyhow!("Peer provided incorrect passphrase."));
        }

        Ok((connection, receive, send))
    }
}

pub enum EthersyncNodeAction {
    Connect { secret_address: SecretAddress },
}

pub enum EthersyncNodeEvent {
    NewConnection {
        connection: iroh::endpoint::Connection,
        receive: RecvStream,
        send: SendStream,
    },
    NodeSpawned {
        node_info: EthersyncNodeInfo,
    },
}

pub async fn create_node_handler(
    mut action_rx: UnboundedReceiver<EthersyncNodeAction>,
    mut event_tx: UnboundedSender<EthersyncNodeEvent>,
) {
    let node = EthersyncNode::spawn().await;

    // TODO: find out how to do this
    /*
    let mut incoming_event_tx = event_tx.clone();
    spawn(async move {
        while let Ok((connection, receive, send)) = node.accept().await {
            incoming_event_tx.send(EthersyncNodeEvent::NewConnection { connection, receive, send }).await.expect("failed to send event!");
        }
    });
     */

    event_tx
        .send(EthersyncNodeEvent::NodeSpawned {
            node_info: node.node_info(),
        })
        .await
        .expect("failed to send event!");

    while let Some(action) = action_rx.next().await {
        match action {
            EthersyncNodeAction::Connect { secret_address } => {
                let result = node.connect(secret_address).await;
                if let Ok((connection, receive, send)) = result {
                    event_tx
                        .send(EthersyncNodeEvent::NewConnection {
                            connection,
                            receive,
                            send,
                        })
                        .await
                        .expect("failed to send event!");
                }
            }
        }
    }
}
