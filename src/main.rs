use async_std::task::sleep;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use dioxus_router::prelude::*;

mod shared;
use shared::ethersync_node::EthersyncNode;

mod ui;
use crate::shared::automerge_document::{AutomergeDocument, FormattedAutomergeMessage};
use crate::ui::file_content_view::FileContentView;
use crate::ui::file_list::FileList;
use ui::automerge_messages_view::AutomergeMessagesView;
use ui::connection_form::ConnectionForm;
use ui::connection_view::ConnectionView;
use ui::node_view::NodeView;
use crate::shared::secret_address::SecretAddress;

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

        Router::<Route> {}
    }
}

#[derive(Routable, Clone)]
enum Route {
    #[route("/?:peer_node_id&:passphrase")]
    EthersyncWeb {
        peer_node_id: String,
        passphrase: String,
    },
}

#[component]
pub fn EthersyncWeb(peer_node_id: String, passphrase: String) -> Element {
    tracing::info!("query: peer_node_id={peer_node_id} passphrase={passphrase}");

    let node = use_resource(|| async { EthersyncNode::spawn().await });

    let mut connection = use_signal(|| None);

    let connect_to_peer = move |secret_address: SecretAddress| async move {
        (&*node.read()).as_ref().expect("Node is not spawned");
        if let Some(node_ref) = &*node.read() {
            let new_connection = node_ref.connect(secret_address).await;
            connection.set(Some(new_connection));
        }
    };

    let mut automerge_messages: Signal<Vec<FormattedAutomergeMessage>> = use_signal(|| Vec::new());

    let mut doc = use_signal(|| {
        AutomergeDocument::default()
        // TODO: load content from local storage?
    });

    let mut selected_file_name = use_signal(|| None);

    let select_file = move |file_name: String| selected_file_name.set(Some(file_name));

    let selected_file_content = selected_file_name
        .read()
        .clone()
        .and_then(|file_name| doc.read().file_content(file_name))
        .unwrap_or_default();

    use_future(move || {
        let secret_address = SecretAddress::from_string(peer_node_id.clone(), passphrase.clone());
        async move {
            if secret_address.is_ok() {
                // remove query parameters to hide passphrase from address bar
                document::eval("history.replaceState(null, null, location.pathname)");

                connect_to_peer(secret_address.unwrap()).await;
            }
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
                let (_from_node_id, message) = connection_ref.receive_message().await;
                let formatted_message = FormattedAutomergeMessage::new("received", &message);
                automerge_messages.push(formatted_message);
                let new_doc = doc.read().apply_message(message).await;
                *doc.write() = new_doc;
            }
        }
    });

    use_future(move || async move {
        // TODO: can this loop be prevented?
        while (&*connection.read()).is_none() {
            sleep(std::time::Duration::from_millis(100)).await;
        }

        if let Some(connection_ref) = &*connection.read() {
            while let Some(message) = doc.read().create_message().await {
                let formatted_message = FormattedAutomergeMessage::new("sent", &message);
                connection_ref.send_message(message).await;
                automerge_messages.push(formatted_message);
            }
        }
    });

    rsx! {
        h1 { "Ethersync-Web" }

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
                            connect_to_peer,
                        }
                    },
                    Some(c) =>  rsx! {
                        ConnectionView {
                            remote_node_id: c.remote_node_id().map(|n| n.to_string())
                        }
                    }
                }
            }
        }

        FileList {
            files: doc.read().files(),
            select_file
        }

        FileContentView {
            file_name: selected_file_name.read().clone().unwrap_or_default(),
            content: selected_file_content
        }

        if !automerge_messages.is_empty() {
            AutomergeMessagesView {
                automerge_messages
            }
        }
    }
}
