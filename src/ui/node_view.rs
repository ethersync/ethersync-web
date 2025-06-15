#![feature(fn_traits)]

use dioxus::prelude::*;
use std::future::Future;

#[derive(PartialEq, Props, Clone)]
pub struct NodeViewProps {
    node_id: Option<String>,
    secret_key: String,
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
        }
    }
}
