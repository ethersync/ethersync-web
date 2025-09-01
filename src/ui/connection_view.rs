use dioxus::prelude::*;
use iroh::NodeId;

#[derive(PartialEq, Props, Clone)]
pub struct ConnectionViewProps {
    connected_peers: Vec<NodeId>,
}

#[component]
pub fn ConnectionView(props: ConnectionViewProps) -> Element {
    rsx! {
        section {
            h2 { "Bidirectional Connection" }

            dl {
                dt { "remote node IDs:" }
                dd {
                    if props.connected_peers.is_empty() {
                        "not connected"
                    } else {
                        for n in  props.connected_peers {
                            p { "{n}" }
                        }
                    }
                }
            }
        }
    }
}
