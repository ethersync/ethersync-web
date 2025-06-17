use std::str::FromStr;
use anyhow::{bail, Result};
use iroh::{NodeId, SecretKey};

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
            peer_passphrase
        })
    }
}
