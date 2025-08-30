use async_std::stream::StreamExt;
use async_std::task::sleep;
use dioxus::prelude::*;
use std::cell::{Ref, RefCell};

mod shared;
use shared::ethersync_client::EthersyncClientAction;
use shared::ethersync_client::EthersyncClientState;
use shared::ethersync_client::create_client;
use shared::ethersync_node::EthersyncNode;

mod ui;
use crate::shared::automerge_document::{AutomergeDocument, FormattedAutomergeMessage};
use crate::shared::ethersync_node::EthersyncNodeConnection;
use crate::shared::secret_address::{get_secret_address_from_wormhole, SecretAddress};
use crate::ui::file_content_view::FileContentView;
use crate::ui::file_list::FileList;
use ui::automerge_messages_view::AutomergeMessagesView;
use ui::connection_form::ConnectionForm;
use ui::connection_view::ConnectionView;
use ui::node_view::NodeView;
use futures::SinkExt;

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
    let mut client_state = use_signal(|| EthersyncClientState::Initial);

    let client_action_tx = use_coroutine(move | mut client_action_rx: UnboundedReceiver<EthersyncClientAction> | async move {
        let (mut state_rx, mut action_tx) = create_client();

        spawn(async move {
            while let Some(state) = state_rx.next().await {
                *client_state.write() = state.expect("Invalid client state!");
            }
        });

        while let Some(action) = client_action_rx.next().await {
            action_tx.send(action).await;
        }
    });

    use_effect(move || {
        client_action_tx.send(EthersyncClientAction::SpawnNode);
    });

    let connect_to_peer = move |secret_address: SecretAddress| async move {
        client_action_tx.send(EthersyncClientAction::Connect{ secret_address })
    };

    if 1 > 1 {}

    rsx! {
        h1 { "Ethersync-Web" }

        match *client_state.read() {
            EthersyncClientState::Initial => rsx! {
                "Spawning node…"
            },

            EthersyncClientState::NodeSpawned { ref node_info } => rsx! {
                NodeView {
                    node_info: node_info.clone()
                }

                ConnectionForm {
                    connect_to_peer,
                }
            },

            EthersyncClientState::Connected { ref node_info, ref remote_node_id } => rsx! {
                NodeView {
                    node_info: node_info.clone()
                }

                ConnectionView {
                    remote_node_id: remote_node_id.clone()
                }
            }
        }
    }

    /*
    let node = use_resource(|| async { EthersyncNode::spawn().await });
    let mut connection: Signal<Option<RefCell<EthersyncNodeConnection>>> = use_signal(|| None);
    let mut remote_node_id = use_signal(|| None);

    use_effect(move || {
        remote_node_id.set(
            match &*connection.read() {
                Some(connection_ref) =>
                    connection_ref.borrow().remote_node_id()
                        .map( |n| n.to_string()),
                None => None
            }
        );
    });

    let connect_to_peer = move |secret_address: SecretAddress| async move {
        (&*node.read()).as_ref().expect("Node is not spawned");
        if let Some(node_ref) = &*node.read() {
            let new_connection = node_ref.connect(secret_address).await;
            connection.set(Some(RefCell::new(new_connection)));
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

    let accept_connection = use_coroutine(
        move |mut rx: UnboundedReceiver<Resource<EthersyncNode>>| async move {
            while let Some(resource) = rx.next().await {
                if let Some(node) = &*resource.read() {
                    if let Some(new_connection) = node.accept().await {
                        connection.set(Some(RefCell::new(new_connection)));
                    }
                }
            }
        },
    );

    use_effect(move || {
        if node.read().is_none() {
            return;
        }
        accept_connection.send(node);
    });

    let receive_messages = use_coroutine(
        move |mut rx: UnboundedReceiver<Signal<Option<RefCell<EthersyncNodeConnection>>>>| async move {
            while let Some(signal) = rx.next().await {
                if let Some(connection) = &*signal.read() {
                    loop {
                        let (_from_node_id, message) =
                            connection.borrow_mut().receive_message().await;
                        let formatted_message =
                            FormattedAutomergeMessage::new("received", &message);
                        automerge_messages.push(formatted_message);
                        let new_doc = doc.read().apply_message(message).await;
                        *doc.write() = new_doc;
                    }
                }
            }
        },
    );

    use_effect(move || {
        if connection.read().is_none() {
            return;
        }
        receive_messages.send(connection);
    });

    let send_sync_messages = use_coroutine(
        move |mut rx: UnboundedReceiver<Signal<Option<RefCell<EthersyncNodeConnection>>>>| async move {
            while let Some(signal) = rx.next().await {
                if let Some(connection) = &*signal.read() {
                    while let Some(message) = doc.read().create_message().await {
                        let formatted_message = FormattedAutomergeMessage::new("sent", &message);
                        connection.borrow_mut().send_message(message).await;
                        automerge_messages.push(formatted_message);
                    }
                }
            }
        },
    );

    use_effect(move || {
        if connection.read().is_none() {
            return;
        }
        send_sync_messages.send(connection);
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
                    my_passphrase: n.my_passphrase.to_string(),
                    secret_key: n.secret_key.to_string()
                }

                match &*connection.read() {
                    None => rsx! {
                        ConnectionForm {
                            connect_to_peer,
                        }
                    },
                    Some(connection_ref) =>  rsx! {
                        ConnectionView {
                            remote_node_id
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
     */
}
