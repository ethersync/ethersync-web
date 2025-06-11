use dioxus::prelude::*;
use futures::StreamExt;
use std::str::FromStr;

const ALPN: &[u8] = b"/iroh-web/0";

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
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

fn generate_random_secret_key() -> iroh::SecretKey {
    iroh::SecretKey::generate(rand::thread_rng())
}

#[component]
pub fn Chat() -> Element {
    let mut secret_key = use_signal(|| generate_random_secret_key());
    let mut endpoint: Signal<Option<iroh::Endpoint>> = use_signal(|| None);

    let mut node_id = match &*endpoint.read() {
        Some(e) => e.node_id().to_string(),
        None => "not connected".to_string(),
    };

    rsx! {
        p {
            style: "display: flex; gap: 1em;",

            label {
                for: "secret_key",
                "secret key:"
            }

            input {
                id: "secret_key",
                value: "{secret_key}",
                oninput: move |event| secret_key.set(iroh::SecretKey::from_str(&event.value()).expect("Invalid key!")),
                style: "min-width: 40em;"
            }

            button {
                onclick: move |_| {
                    secret_key.set(generate_random_secret_key());
                },
                "generate"
            }

            button {
                onclick: move |_| async move {
                    endpoint.set(Some(
                        iroh::Endpoint::builder()
                            .secret_key(secret_key.read().clone())
                            .alpns(vec![ALPN.to_vec()])
                            .discovery_n0()
                            .bind()
                            .await
                            .unwrap()
                    ));
                },
                "connect"
            }
        }

        p { "node id: {node_id}" }
    }
}
