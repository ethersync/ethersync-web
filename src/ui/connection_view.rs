use crate::services::connection_service::{CONNECTED_PEERS, CONNECTION_EVENTS};
use dioxus::prelude::*;

#[component]
pub fn ConnectionView() -> Element {
    rsx! {
        section {
            h2 { "Connected Peers" }

            if CONNECTED_PEERS.is_empty() {
                p { "not connected" }
            } else {
                ul {
                    for node_id in CONNECTED_PEERS.iter() {
                        li { "{node_id}" }
                    }
                }
            }

            hr { }

            ul {
                for event in CONNECTION_EVENTS.iter() {
                    li { "{event}" }
                }
            }
        }
    }
}
