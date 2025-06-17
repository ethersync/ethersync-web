use anyhow::{bail, Result};
use iroh::{NodeId, SecretKey};
use magic_wormhole::{transfer, AppID, Code, MailboxConnection, Wormhole};
use std::str::FromStr;

pub struct SecretAddress {
    pub peer_node_id: NodeId,
    pub peer_passphrase: SecretKey,
}

impl SecretAddress {
    pub fn from_string(peer_node_id: String, peer_passphrase: String) -> Result<Self> {
        if peer_node_id.is_empty() {
            bail!("peer_node_id is empty!")
        }

        if peer_passphrase.is_empty() {
            bail!("peer_passphrase is empty!")
        }

        let peer_node_id = NodeId::from_str(&peer_node_id)?;
        let peer_passphrase = SecretKey::from_str(&peer_passphrase)?;

        Ok(Self {
            peer_node_id,
            peer_passphrase,
        })
    }
}

pub async fn get_secret_address_from_wormhole(code: &str) -> Result<SecretAddress> {
    let config = transfer::APP_CONFIG.id(AppID::new("ethersync"));

    let mut wormhole =
        Wormhole::connect(MailboxConnection::connect(config, Code::from_str(code)?, false).await?)
            .await?;
    let bytes = wormhole.receive().await?;
    let fragments: Vec<String> = String::from_utf8(bytes)?
        .clone()
        .split("#")
        .map(|value| value.to_string())
        .collect();
    let peer_node_id = fragments[0].to_string();
    let peer_passphrase = fragments[1].to_string();
    Ok(SecretAddress::from_string(peer_node_id, peer_passphrase)?)
}
