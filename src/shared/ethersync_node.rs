use automerge::sync::{Message as AutomergeSyncMessage};
use dioxus::logger::tracing;
use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::{Endpoint, NodeId, SecretKey};
use std::cell::RefCell;
use crate::shared::secret_address::SecretAddress;

const ALPN: &[u8] = b"/ethersync/0";

pub struct EthersyncNode {
    pub endpoint: Endpoint,
    pub my_passphrase: SecretKey,
    pub secret_key: SecretKey,
}

fn generate_random_secret_key() -> SecretKey {
    SecretKey::generate(rand::thread_rng())
}

impl EthersyncNode {
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
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

    pub async fn connect(&self, secret_address: SecretAddress) -> EthersyncNodeConnection {
        let connection = self
            .endpoint
            .connect(secret_address.peer_node_id, ALPN)
            .await
            .expect("Failed to connect!");

        let (mut send, receive) = connection.open_bi().await.expect("Failed to bi!");

        send.write_all(&secret_address.peer_passphrase.to_bytes())
            .await
            .expect("Failed to send peer passphrase!");

        EthersyncNodeConnection {
            connection,
            receive: RefCell::new(receive),
            send: RefCell::new(send),
        }
    }

    pub async fn accept(&self) -> Option<EthersyncNodeConnection> {
        let incoming = self.endpoint.accept().await;
        if incoming.is_none() {
            return None;
        }

        let connection = incoming.unwrap().await.expect("Failed to connect!");
        let (send, mut receive) = connection.accept_bi().await.expect("Failed to accept!");

        let mut received_passphrase = [0; 32];
        receive
            .read_exact(&mut received_passphrase)
            .await
            .expect("Failed to receive passphrase!");

        // Guard against timing attacks.
        if !constant_time_eq::constant_time_eq(&received_passphrase, &self.my_passphrase.to_bytes())
        {
            tracing::warn!("Peer provided incorrect passphrase.");
            return None;
        }

        Some(EthersyncNodeConnection {
            connection,
            receive: RefCell::new(receive),
            send: RefCell::new(send),
        })
    }
}

pub struct EthersyncNodeConnection {
    connection: Connection,
    receive: RefCell<RecvStream>,
    send: RefCell<SendStream>,
}

impl EthersyncNodeConnection {
    pub fn remote_node_id(&self) -> Option<NodeId> {
        self.connection.remote_node_id().ok()
    }

    pub async fn send_message(&self, message: AutomergeSyncMessage) {
        let message_buf = message.encode();
        let message_len =
            u32::try_from(message_buf.len()).expect("Failed to convert message length!");
        self.send
            .borrow_mut()
            .write_all(&message_len.to_be_bytes())
            .await
            .expect("Failed to send message length!");

        self.send
            .borrow_mut()
            .write_all(&message_buf)
            .await
            .expect("Failed to send message!");
    }

    pub async fn receive_message(&self) -> (NodeId, AutomergeSyncMessage) {
        let mut message_len_buf = [0; 4];
        self.receive
            .borrow_mut()
            .read_exact(&mut message_len_buf)
            .await
            .expect("Failed to message length!");
        let message_len = u32::from_be_bytes(message_len_buf);

        let mut message_buf = vec![0; message_len as usize];
        self.receive
            .borrow_mut()
            .read_exact(&mut message_buf)
            .await
            .expect("Failed to message!");

        let message =
            AutomergeSyncMessage::decode(&message_buf).expect("Failed to parse automerge message!");

        let from_node_id = self.remote_node_id().expect("Missing remote node ID!");

        (from_node_id, message)
    }
}
