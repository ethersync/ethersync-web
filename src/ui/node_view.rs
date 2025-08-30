use dioxus::prelude::*;
use crate::shared::ethersync_node::EthersyncNodeInfo;

#[derive(PartialEq, Props, Clone)]
pub struct NodeViewProps {
    node_info: EthersyncNodeInfo
}

#[component]
pub fn NodeView(props: NodeViewProps) -> Element {
    rsx! {
        section {
            h2 { "Node" }

            dl {
                dt { "secret key:" }
                dd { "{props.node_info.secret_key}" }

                dt { "node ID:" }
                dd { "{props.node_info.node_id}" }

                dt { "Ethersync passphrase:" }
                dd { "{props.node_info.my_passphrase}" }
            }
        }
    }
}
