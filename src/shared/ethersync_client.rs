use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::sync::Arc;
use crate::shared::ethersync_node::{EthersyncNode, EthersyncNodeConnection, EthersyncNodeInfo};
use crate::shared::secret_address::SecretAddress;
use anyhow::{anyhow, Result};
use dioxus::prelude::spawn;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use iroh::endpoint::RecvStream;
use iroh::NodeId;
use crate::shared::automerge_document::AutomergeDocument;
use automerge::sync::{Message as AutomergeSyncMessage};
use dioxus::dioxus_core::SpawnIfAsync;

#[derive(Clone)]
pub enum EthersyncClientEvent {
    NodeSpawned { node_info: EthersyncNodeInfo },
    Connected { remote_node_id: Option<String> },
}

pub enum EthersyncClientAction {
    SpawnNode,
    Connect { secret_address: SecretAddress },
}

type EthersyncClientEventReceiver =
    mpsc::Receiver<Result<EthersyncClientEvent>>;
type EthersyncClientActionSender = mpsc::Sender<EthersyncClientAction>;


pub struct EthersyncClient {
    node: Option<EthersyncNode>,
    connection: Option<EthersyncNodeConnection>,
}

impl EthersyncClient {
    pub fn create() -> (EthersyncClientEventReceiver, EthersyncClientActionSender) {
        let mut client = Self {
            node: None,
            connection: None,
        };

        let (action_tx, mut action_rx) = mpsc::channel(1);
        let (mut event_tx, event_rx) = mpsc::channel(1);

        spawn(async move {
            while let Some(action) = action_rx.next().await {
                let event = client.handle_action(action).await;
                event_tx.send(event).await.expect("Could not send result!");
                
                if let Some( c) = client.connection.as_mut() {
                    spawn(async move {
                       c.receive_message() .await;
                    });
                }
            }
        });

        (event_rx, action_tx)
    }

    async fn handle_action(&mut self, action: EthersyncClientAction) -> Result<EthersyncClientEvent> {
        match action {
            EthersyncClientAction::SpawnNode => Ok(self.spawn_node().await?),
            EthersyncClientAction::Connect  { secret_address } => Ok(self.create_connection(secret_address).await?),
        }
    }

    async fn create_connection(&mut self, secret_address: SecretAddress) -> Result<EthersyncClientEvent> {
        if self.node.is_none() {
            return Err(anyhow!("not spawned"))
        }

        if self.connection.is_some() {
            return Err(anyhow!("already connected"))
        }

        let node_ref = self.node.as_ref().unwrap();
        let mut new_connection = node_ref.connect(secret_address).await?;

        let message_rx = new_connection.message_rx.borrow_mut();
        
        self.connection = Some(new_connection);
        let remote_node_id = self.connection
            .as_ref()
            .and_then(|c| c.remote_node_id())
            .map(|n| n.to_string());

        spawn(async move {
            // let (_from_node_id, message) =c.receive_message().await
                while message_rx.next().await.is_some() {

                }
        });

        Ok(EthersyncClientEvent::Connected { remote_node_id })
    }

    async fn spawn_node(&mut self) -> Result<EthersyncClientEvent> {
        if self.node.is_some() {
            return Err(anyhow!("already spawned"))
        }

        self.node = Some(EthersyncNode::spawn().await);
        Ok(EthersyncClientEvent::NodeSpawned {
            node_info: self.node.as_ref().map(|n| n.node_info()).unwrap(),
        })
    }
}

pub fn create_client() -> (EthersyncClientEventReceiver, EthersyncClientActionSender) {
    let (action_tx, mut action_rx) = mpsc::channel(1);
    let (mut state_tx, state_rx) = mpsc::channel(1);

    spawn(async move {
        let mut node = None;
        let mut connection = None;

        // TODO: load content from local storage?
        let mut doc = AutomergeDocument::default();

        while let Some(action) = action_rx.next().await {
            let result = match action {
                EthersyncClientAction::SpawnNode => {
                    if node.is_some() {
                        Err(anyhow!("already spawned"))
                    } else {
                        node = Some(EthersyncNode::spawn().await);
                        Ok(EthersyncClientEvent::NodeSpawned {
                            node_info: node.as_ref().expect("Node is not spawned!").node_info(),
                        })
                    }
                }
                EthersyncClientAction::Connect { secret_address } => {
                    if node.is_none() {
                        Err(anyhow!("not spawned"))
                    } else if connection.is_some() {
                        Err(anyhow!("already connected"))
                    } else {
                        let node_ref = node.as_ref().unwrap();
                        match node_ref.connect(secret_address).await {
                            // TODO: forward error
                            Err(error) => Err(error),
                            Ok(new_connection) => {
                                connection = Some(RefCell::new(new_connection));
                                let remote_node_id = connection
                                    .as_ref()
                                    .and_then(|c| c.borrow().remote_node_id())
                                    .map(|n| n.to_string());

                                // TODO: spawn and receive_message

                                Ok(EthersyncClientEvent::Connected { remote_node_id })
                            }
                        }
                    }
                }
            };

            state_tx.send(result).await.expect("Could not send result!");
        }
    });

    (state_rx, action_tx)
}
