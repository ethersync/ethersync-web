use crate::services::node_service::{NODE_EVENTS, NODE_INFO};
use dioxus::prelude::*;

#[component]
pub fn NodeInfoView() -> Element {
    rsx! {
        section {
            h2 { "Node" }

            match NODE_INFO.as_ref() {
                None => rsx! {
                    "Spawning node…"
                },
                Some(node_info) =>  rsx! {
                    dl {
                        dt { "secret key:" }
                        dd { "{node_info.secret_key}" }

                        dt { "node ID:" }
                        dd { "{node_info.node_id}" }

                        dt { "Ethersync passphrase:" }
                        dd { "{node_info.my_passphrase}" }
                    }
                }
            }

            hr { }

            ul {
                for event in NODE_EVENTS.iter() {
                    li { "{event}" }
                }
            }
        }
    }
}
