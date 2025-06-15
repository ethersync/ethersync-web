use std::cell::RefCell;
use std::str::FromStr;
use iroh::{Endpoint, NodeId, SecretKey};
use iroh::endpoint::{Connection, RecvStream, SendStream};

const ALPN: &[u8] = b"/iroh-web/0";
const HELLO: &[u8] = b"Hello!";

pub struct ChatNode {
    pub endpoint: Endpoint,
    pub secret_key: SecretKey,
}

fn generate_random_secret_key() -> SecretKey {
    SecretKey::generate(rand::thread_rng())
}

impl ChatNode {
    pub fn node_id(&self) -> NodeId {
        self.endpoint.node_id()
    }

    pub async fn spawn() -> Self {
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
            secret_key
        }
    }

    pub async fn connect(&self, peer_node_id: String) -> ChatNodeConnection {
        let node_addr: iroh::NodeAddr = NodeId::from_str(&peer_node_id).expect("Invalid node id!").into();
        let connection = self.endpoint.connect(node_addr, crate::ALPN).await.expect("Failed to connect!");
        let (mut send, mut receive) = connection.open_bi().await.expect("Failed to bi!");

        send.write(HELLO).await.expect("Failed to send hello!");

        let mut buffer = vec![0; HELLO.len()];
        receive.read_exact(&mut buffer).await.expect("Failed to receive hello!");

        ChatNodeConnection {
            connection,
            receive: RefCell::new(receive),
            send: RefCell::new(send),
        }
    }

    pub async fn accept(&self) -> Option<ChatNodeConnection> {
        let incoming = self.endpoint.accept().await;
        if incoming.is_none() {
            return None;
        }

        let connection = incoming.unwrap().await.expect("Failed to connect!");
        let (mut send, mut receive) =
            connection.accept_bi().await.expect("Failed to accept!");

        let mut buffer = vec![0; HELLO.len()];
        receive.read_exact(&mut buffer).await.expect("Failed to receive hello!");

        send.write(HELLO).await.expect("Failed to send hello!");

        Some(ChatNodeConnection {
            connection,
            receive: RefCell::new(receive),
            send: RefCell::new(send)
        })
    }
}

pub struct ChatNodeConnection {
    pub connection: Connection,
    pub receive: RefCell<RecvStream>,
    pub send: RefCell<SendStream>,
}

impl ChatNodeConnection {
    pub fn remote_node_id(&self) -> Option<NodeId> {
        self.connection.remote_node_id().ok()
    }
}
