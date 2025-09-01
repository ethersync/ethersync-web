use async_std::stream::StreamExt;
use dioxus::prelude::*;

mod shared;

mod ui;
use crate::shared::automerge_document::{
    AutomergeDocumentAction, AutomergeDocumentEvent, AutomergeDocumentHandler,
    FormattedAutomergeMessage,
};
use crate::shared::ethersync_connection::{
    create_connection_handler, EthersyncConnectionAction, EthersyncConnectionEvent,
};
use crate::shared::ethersync_node::{create_node_handler, EthersyncNodeAction, EthersyncNodeEvent};
use crate::shared::secret_address::{get_secret_address_from_wormhole, SecretAddress};
use crate::ui::file_content_view::FileContentView;
use crate::ui::file_list::FileList;
use ui::automerge_messages_view::AutomergeMessagesView;
use ui::connection_form::ConnectionForm;
use ui::connection_view::ConnectionView;
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

        Router::<Route> {}
    }
}

#[derive(Routable, Clone)]
enum Route {
    #[route("/?:join_code")]
    EthersyncWeb { join_code: String },
}

#[component]
pub fn EthersyncWeb(join_code: String) -> Element {
    let mut node_info_signal = use_signal(|| None);
    let mut connected_peers_signal = use_signal(Vec::new);
    let mut automerge_messages: Signal<Vec<FormattedAutomergeMessage>> = use_signal(Vec::new);
    let mut files_signal = use_signal(Vec::new);
    let mut selected_file_signal = use_signal(|| None);

    let doc_event_tx = use_coroutine(move |mut doc_event_rx| async move {
        while let Some(event) = doc_event_rx.next().await {
            match event {
                AutomergeDocumentEvent::Changed {
                    files,
                    selected_file,
                } => {
                    files_signal.set(files);
                    selected_file_signal.set(selected_file);
                }
            }
        }
    });

    let doc_action_tx = use_coroutine(move |doc_action_rx| async move {
        let mut handler = AutomergeDocumentHandler {
            doc_event_tx: doc_event_tx.tx(),
        };
        handler.handle_actions(doc_action_rx).await;
    });

    let select_file = use_callback(move |file_name| {
        doc_action_tx.send(AutomergeDocumentAction::SelectFile { file_name })
    });

    let connection_event_tx = use_coroutine(move |mut connection_event_rx| async move {
        while let Some(event) = connection_event_rx.next().await {
            match event {
                EthersyncConnectionEvent::Connected {
                    remote_node_id,
                    outgoing_message_tx,
                } => {
                    connected_peers_signal.write().push(remote_node_id);
                    doc_action_tx.send(AutomergeDocumentAction::Sync {
                        outgoing_message_tx,
                    });
                }
                EthersyncConnectionEvent::MessageReceived {
                    remote_node_id: _,
                    message,
                } => {
                    let formatted_message = FormattedAutomergeMessage::new("received", &message);
                    automerge_messages.push(formatted_message);
                    doc_action_tx.send(AutomergeDocumentAction::Apply { message });
                }
                EthersyncConnectionEvent::MessageSent { message } => {
                    let formatted_message = FormattedAutomergeMessage::new("sent", &message);
                    automerge_messages.push(formatted_message);
                }
            }
        }
    });

    let connection_action_tx = use_coroutine(move |connection_action_rx| async move {
        create_connection_handler(connection_action_rx, connection_event_tx.tx()).await;
    });

    let node_event_tx = use_coroutine(move |mut node_event_rx| async move {
        while let Some(event) = node_event_rx.next().await {
            match event {
                EthersyncNodeEvent::NewConnection {
                    connection,
                    receive,
                    send,
                } => connection_action_tx.send(EthersyncConnectionAction::AddConnection {
                    connection,
                    receive,
                    send,
                }),
                EthersyncNodeEvent::NodeSpawned { node_info } => {
                    node_info_signal.set(Some(node_info));
                }
            }
        }
    });

    let node_action_tx = use_coroutine(move |node_action_rx| async move {
        create_node_handler(node_action_rx, node_event_tx.tx()).await;
    });

    let connect_to_peer = move |secret_address: SecretAddress| async move {
        node_action_tx.send(EthersyncNodeAction::Connect { secret_address });
    };

    use_future(move || {
        let join_code = join_code.clone();
        async move {
            if !join_code.clone().is_empty() {
                let secret_address = get_secret_address_from_wormhole(&join_code)
                    .await
                    .expect("Invalid secret address!");
                connect_to_peer(secret_address).await;
            }
        }
    });

    rsx! {
        h1 { "Ethersync-Web" }

        match &*node_info_signal.read() {
            None => rsx! {
                "Spawning node…"
            },
            Some(node_info) => rsx! {
                NodeView {
                    node_info: node_info.clone()
                }

                ConnectionForm {
                    connect_to_peer,
                }
            }
        }

        ConnectionView {
            connected_peers: connected_peers_signal.read().clone()
        }

        FileList {
            files: files_signal.read().clone(),
            select_file
        }

        match selected_file_signal.read().clone() {
            Some(selected_file) => rsx! {
                FileContentView {
                    selected_file
                }
            },
            None => rsx! {}
        }

        if !automerge_messages.is_empty() {
            AutomergeMessagesView {
                automerge_messages
            }
        }
    }
}
