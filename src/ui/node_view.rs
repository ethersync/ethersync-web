use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct NodeViewProps {
    node_id: Option<String>,
    my_passphrase: String,
    secret_key: String,
}

#[component]
pub fn NodeView(props: NodeViewProps) -> Element {
    rsx! {
        section {
            h2 { "Node" }

            dl {
                dt { "secret key:" }
                dd { "{props.secret_key}" }

                dt { "node ID:" }
                dd {
                    match props.node_id {
                        Some(n) => rsx! { "{n}" },
                        None => rsx! { "not connected" }
                    }
                }

                dt { "Ethersync passphrase:" }
                dd { "{props.my_passphrase}" }
            }
        }
    }
}
