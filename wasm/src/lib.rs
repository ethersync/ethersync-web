mod utils;

use wasm_bindgen::prelude::*;

const ALPN: &[u8] = b"/iroh-web/0";

#[wasm_bindgen]
pub struct IrohNode {
    endpoint: iroh::Endpoint
}

#[wasm_bindgen]
impl IrohNode {
    pub async fn connect() -> Self {
        let secret_key = iroh::SecretKey::generate(rand::thread_rng());
        // let secret_key = iroh::SecretKey::generate(rand::rngs::OsRng);
        let endpoint = iroh::Endpoint::builder()
            .secret_key(secret_key)
            .alpns(vec![ALPN.to_vec()])
            .discovery_n0()
            .bind()
            .await
            .expect("Failed to connect!");

        Self {
            endpoint
        }
    }

    // we cannot use iroh::NodeId here because it does not compile to WASM
    pub fn node_id(&self) -> String {
        self.endpoint.node_id().to_string()
    }
}
