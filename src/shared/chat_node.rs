use iroh::{Endpoint, NodeId, SecretKey};

const ALPN: &[u8] = b"/iroh-web/0";

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
}
