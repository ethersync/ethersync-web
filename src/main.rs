use async_std::task::sleep;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use futures::StreamExt;
use std::str::FromStr;

mod shared;
use shared::chat_node::ChatNode;

mod ui;
use ui::node_view::NodeView;

const ALPN: &[u8] = b"/iroh-web/0";

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::logger::initialize_default();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Chat {}

    }
}

#[component]
pub fn Chat() -> Element {
    let mut node = use_signal(|| None);

    let spawn_node = move || async move {
        node.set(Some(ChatNode::spawn().await));
    };

    use_future(move || async move {
        spawn_node().await
    });

    let mut peer_node_id = use_signal(|| "".to_string());
    let mut message_text = use_signal(|| "".to_string());
    let mut messages = use_signal(|| Vec::new());

    use_future(move || async move {
        let mut hash_value = document::eval("return location.hash")
            .await
            .unwrap()
            .as_str()
            .unwrap()
            .trim_start_matches("#")
            .to_string();
        if !hash_value.is_empty() {
            peer_node_id.set(hash_value)
        }
    });

    use_future(move || async move {
        if (&*node.read()).is_none() {
            tracing::info!("node is none");
            return
        }

        tracing::info!("node is some")
    });

    let chat_client = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        while (&*node.read()).is_none() {
            sleep(std::time::Duration::from_millis(100)).await;
        }

        if let Some(node_ref) = &*node.read() {
            let endpoint = &node_ref.endpoint;
            while let Some(incoming) = endpoint.accept().await {
                let connection = incoming.await.expect("Failed to connect!");
                let (mut send, mut receive) =
                    connection.accept_bi().await.expect("Failed to accept!");
                send.write_all("unused".as_bytes())
                    .await
                    .expect("Failed to send!");
                send.finish().expect("Failed to finish!");
                let received_bytes = receive.read_to_end(1000).await.expect("Failed to read!");
                let message = str::from_utf8(&received_bytes).expect("Failed to parse message!");
                let from_node_id = connection
                    .remote_node_id()
                    .expect("Missing remote node ID!");
                messages.push(format!("{message} from {from_node_id}"));

                if (&*peer_node_id.read()).is_empty() {
                    peer_node_id.set(from_node_id.to_string());
                }
            }
        }
    });

    rsx! {
        h1 { "iroh Chat" }

        match &*node.read() {
            Some(n) => rsx! {
                NodeView {
                    node_id: n.node_id().to_string(),
                    rotate_secret_key: spawn_node,
                    secret_key: n.secret_key.to_string()
                }

                section {
                    style: "display: flex; gap: 1em;",

                    label {
                        for: "peer_node_id",
                        "peer node id:"
                    }

                    input {
                        id: "peer_node_id",
                        value: "{peer_node_id}",
                        oninput: move |event| peer_node_id.set(event.value().clone()),
                        style: "min-width: 40em;"
                    }
                }

                section {
                    style: "display: flex; gap: 1em;",

                    label {
                        for: "message_text",
                        "text:"
                    }

                    input {
                        id: "message_text",
                        value: "{message_text}",
                        oninput: move |event| message_text.set(event.value().clone()),
                        style: "min-width: 40em;"
                    }

                    button {
                        disabled: (&*peer_node_id.read()).is_empty() || (&*message_text.read()).is_empty(),
                        onclick: move |_| async move {
                            let node_addr: iroh::NodeAddr = iroh::NodeId::from_str(&peer_node_id.read().clone()).expect("Invalid node id!").into();

                            if let Some(node_ref) = &*node.read() {
                                let endpoint = &node_ref.endpoint;
                                let connection = endpoint.connect(node_addr, ALPN).await.expect("Failed to connect!");
                                let (mut send, mut receive) = connection.open_bi().await.expect("Failed to bi!");
                                send.write_all(message_text.read().as_bytes()).await.expect("Failed to send!");
                                send.finish().expect("Failed to finish!");
                                let _ = receive.read_to_end(1000).await;
                                message_text.set("".to_string());
                            }
                        },
                        "send"
                    }
                }

                ul {
                    for text in &*messages.read() {
                        li { "{text}" }
                    }
                }
            },
            None => rsx! {}
        }
    }
}
