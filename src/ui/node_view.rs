use std::future::Future;
use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct NodeViewProps {
    node_id: Option<String>,
    rotate_secret_key: Callback,
    secret_key: String
}

#[component]
pub fn NodeView(props: NodeViewProps) -> Element {
    rsx! {
        section {
            h2 { "Node" }

            dl {
                dt { "node ID:" }
                dd {
                    match props.node_id {
                        Some(n) => rsx! { "{n}" },
                        None => rsx! { "not connected" }
                    }
                }

                dt { "secret key:" }
                dd { "{props.secret_key}" }
            }

            button {
                onclick: move |_| {
                    props.rotate_secret_key.call(());
                },
                "rotate secret key"
            }
        }
    }
}
