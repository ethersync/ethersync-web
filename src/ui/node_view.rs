use crate::services::node_service::{NODE_ERRORS, NODE_INFO};
use dioxus::prelude::*;

#[component]
pub fn NodeInfoView() -> Element {
    let error_messages: Vec<String> = NODE_ERRORS.iter().map(|error| format!("{error}")).collect();

    rsx! {
        section {
            h2 { "Node" }

            for text in error_messages {
                p { "{text}" }
            }

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
        }
    }
}
