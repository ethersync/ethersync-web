use dioxus::prelude::*;
use iroh_web_shared::IrohNode;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        ConnectNode {}
    }
}

#[component]
fn ConnectNode() -> Element {
    let node = use_resource(|| async {
        IrohNode::connect().await
    });

    /*
    let mut node= use_signal(|| None as Option<IrohNode>);
    let button_label = use_memo(move || match &*node.read() {
        Some(n) => format!("node id: {value}", value = n.node_id()),
        None => "connect".to_string(),
    });

     */

    rsx! {
        match &*node.read_unchecked() {
            Some(_n) => rsx! { p { "n.node_id()" } },
            None =>  rsx! { p { "Loading..." } }
        }
    }
}
