use async_std::task::sleep;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use futures::StreamExt;
use std::cell::RefCell;
use std::str::FromStr;

mod shared;
use shared::chat_node::ChatNode;

mod ui;
use crate::shared::chat_node::ChatNodeConnection;
use ui::connection_form::ConnectionForm;
use ui::connection_view::ConnectionView;
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
    let mut node = use_resource(|| async { ChatNode::spawn().await });

    let mut connection = use_signal(|| None);

    let connect_to_peer = move |peer_node_id: String| async move {
        (&*node.read()).as_ref().expect("Node is not spawned");
        if let Some(node_ref) = &*node.read() {
            let new_connection = node_ref.connect(peer_node_id).await;
            connection.set(Some(new_connection));
        }
    };

    let mut message_text = use_signal(|| "".to_string());
    let mut messages: Signal<Vec<String>> = use_signal(|| Vec::new());

    use_future(move || async move {
        let mut hash_value = document::eval("return location.hash")
            .await
            .unwrap()
            .as_str()
            .unwrap()
            .trim_start_matches("#")
            .to_string();

        if !hash_value.is_empty() {
            connect_to_peer(hash_value).await;
        }
    });

    use_future(move || async move {
        // TODO: can this loop be prevented?
        while (&*node.read()).is_none() {
            sleep(std::time::Duration::from_millis(100)).await;
        }

        if let Some(node_ref) = &*node.read() {
            let new_connection = node_ref.accept().await;
            connection.set(new_connection);
        }
    });

    use_future(move || async move {
        // TODO: can this loop be prevented?
        while (&*connection.read()).is_none() {
            sleep(std::time::Duration::from_millis(100)).await;
        }

        if let Some(connection_ref) = &*connection.read() {
            loop {
                let (from_node_id, message) = connection_ref.receive_message().await;
                messages.push(format!("{message} from {from_node_id}"));
            }
        }
    });

    rsx! {
        h1 { "iroh Chat" }

        match &*node.read() {
            Some(n) => rsx! {
                NodeView {
                    node_id: n.node_id().to_string(),
                    secret_key: n.secret_key.to_string()
                }

                match &*connection.read() {
                    Some(c) =>  rsx! {
                        ConnectionView {
                            remote_node_id: c.remote_node_id().map(|n| n.to_string())
                        }
                    },
                    None => rsx! {
                        ConnectionForm {
                            connect_to_peer: connect_to_peer,
                        }
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
                        disabled: (&*message_text.read()).is_empty(),
                        onclick: move |_| async move {

                            if let Some(connection_ref) = &*connection.read() {
                                let message =( &*message_text.read()).clone();
                                connection_ref.send_message(message).await;
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
