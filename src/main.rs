use async_std::task::sleep;
use dioxus::prelude::*;

mod shared;
use shared::chat_node::ChatNode;

mod ui;
use ui::connection_form::ConnectionForm;
use ui::connection_view::ConnectionView;
use ui::incoming_messages_view::IncomingMessagesView;
use ui::new_message_form::NewMessageForm;
use ui::node_view::NodeView;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

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
    let node = use_resource(|| async { ChatNode::spawn().await });

    let mut connection = use_signal(|| None);

    let connect_to_peer = move |peer_node_id: String| async move {
        (&*node.read()).as_ref().expect("Node is not spawned");
        if let Some(node_ref) = &*node.read() {
            let new_connection = node_ref.connect(peer_node_id).await;
            connection.set(Some(new_connection));
        }
    };

    let send_message = move |new_message: String| async move{
        (&*connection.read()).as_ref().expect("Node is not connected");
        if let Some(connection_ref) = &*connection.read() {
            connection_ref.send_message(new_message).await;
        }
    };

    let mut incoming_messages: Signal<Vec<String>> = use_signal(|| Vec::new());

    use_future(move || async move {
        let hash_value = document::eval("return location.hash")
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
                incoming_messages.push(format!("{message} from {from_node_id}"));
            }
        }
    });

    rsx! {
        h1 { "iroh Chat" }

        match &*node.read() {
            None => rsx! {
                "Spawning node…"
            },
            Some(n) => rsx! {
                NodeView {
                    node_id: n.node_id().to_string(),
                    secret_key: n.secret_key.to_string()
                }

                match &*connection.read() {
                    None => rsx! {
                        ConnectionForm {
                            connect_to_peer: connect_to_peer,
                        }
                    },
                    Some(c) =>  rsx! {
                        ConnectionView {
                            remote_node_id: c.remote_node_id().map(|n| n.to_string())
                        }

                        NewMessageForm {
                            send_message
                        }

                        IncomingMessagesView {
                            incoming_messages
                        }
                    }
                }
            }
        }
    }
}
