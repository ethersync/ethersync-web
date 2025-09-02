use crate::services::connection_service::{CONNECTED_PEERS, CONNECTION_ERRORS};
use dioxus::prelude::*;

#[component]
pub fn ConnectionView() -> Element {
    let error_messages: Vec<String> = CONNECTION_ERRORS
        .iter()
        .map(|error| format!("{error}"))
        .collect();
    let connected_peers = CONNECTED_PEERS.read().to_owned();

    rsx! {
        section {
            h2 { "Bidirectional Connection" }

            for text in error_messages {
                p { "{text}" }
            }

            dl {
                dt { "remote node IDs:" }
                dd {
                    if connected_peers.is_empty() {
                        "not connected"
                    } else {
                        for n in connected_peers {
                            p { "{n}" }
                        }
                    }
                }
            }
        }
    }
}
