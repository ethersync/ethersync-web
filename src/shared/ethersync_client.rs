use std::cell::RefCell;
use crate::shared::ethersync_node::{EthersyncNode, EthersyncNodeInfo};
use dioxus::prelude::spawn;
use futures::channel::mpsc;
use futures::{FutureExt, SinkExt, StreamExt};
use crate::shared::secret_address::SecretAddress;

#[derive(Clone)]
pub enum EthersyncClientState {
    Initial,
    NodeSpawned { 
        node_info: EthersyncNodeInfo
    },
    Connected {
        node_info: EthersyncNodeInfo,
        remote_node_id: Option<String>
    },
}

pub enum EthersyncClientAction {
    SpawnNode,
    Connect { secret_address: SecretAddress },
}

#[derive(Debug)]
pub struct InvalidStateError;

type EthersyncClientStateReceiver = mpsc::Receiver<Result<EthersyncClientState, InvalidStateError>>;
type EthersyncClientActionSender = mpsc::Sender<EthersyncClientAction>;

pub fn create_client() -> (EthersyncClientStateReceiver, EthersyncClientActionSender) {
    let (action_tx, mut action_rx) = mpsc::channel(1);
    let (mut state_tx, state_rx) = mpsc::channel(1);

    spawn(async move {
        let mut state = EthersyncClientState::Initial;
        let mut node = None;
        let mut connection = None;
        while let Some(action) = action_rx.next().await {
            match action {
                EthersyncClientAction::SpawnNode => match state {
                    EthersyncClientState::Initial => {
                        node = Some(EthersyncNode::spawn().await);
                        state = EthersyncClientState::NodeSpawned {
                            node_info: node.as_ref().expect("Node is not spawned!").node_info(),
                        };
                    }
                    _ => {
                        state_tx
                            .send(Err(InvalidStateError {}))
                            .await
                            .expect("Could not send client state!");
                        continue;
                    }
                },
                EthersyncClientAction::Connect { secret_address } => match state {
                    EthersyncClientState::NodeSpawned { ref node_info } => {
                        connection = Some(node.as_ref().expect("Node is not spawned!").connect(secret_address).await);
                        state = EthersyncClientState::Connected {
                            node_info: node.as_ref().expect("Node is not spawned!").node_info(),
                            remote_node_id: connection.as_ref().expect("Not connected!").remote_node_id()
                            .map( |n| n.to_string()),
                        };
                    },
                    _ => {
                        state_tx
                            .send(Err(InvalidStateError {}))
                            .await
                            .expect("Could not send client state!");
                        continue;
                    }
                }
            }

            state_tx
                .send(Ok(state.clone()))
                .await
                .expect("Could not send client state!");
        }
    });

    (state_rx, action_tx)
}
