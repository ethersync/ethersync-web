use dioxus::prelude::*;

mod services;
mod ui;

use crate::services::automerge_service::start_automerge_service;
use crate::services::connection_service::start_connection_service;
use crate::services::node_service::{start_node_service, NodeCommand, NODE_INFO};
use crate::ui::automerge_document_view::AutomergeDocumentView;
use crate::ui::file_content_view::FileContentView;
use ui::connection_form::ConnectionForm;
use ui::connection_view::ConnectionView;
use ui::node_view::NodeInfoView;

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
    use_coroutine(start_automerge_service);
    use_coroutine(start_connection_service);
    let node_service = use_coroutine(start_node_service);

    use_effect(move || {
        if join_code.is_empty() {
            return;
        }

        node_service.send(NodeCommand::ConnectByJoinCode {
            join_code: join_code.clone(),
        });
    });

    rsx! {
        h1 { "Ethersync-Web" }

        NodeInfoView { }

        if NODE_INFO.read().is_some() {
            ConnectionForm { }
        }

        ConnectionView { }
        AutomergeDocumentView { }
        FileContentView { }
    }
}
