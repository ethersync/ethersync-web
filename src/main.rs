use dioxus::prelude::*;

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
        SecretKey {}

    }
}

#[component]
pub fn SecretKey() -> Element {
    let mut secret_key = use_signal(|| "".to_string());

    rsx! {
        input {
            value: "{secret_key}",
            oninput: move |event| secret_key.set(event.value()),
            style: "margin-right: 1em; min-width: 40em;"
        }
        button {
            onclick: move |_| {
                let random_key = iroh::SecretKey::generate(rand::thread_rng());
                secret_key.set(random_key.to_string());
            },
            "generate"
        }
    }
}
