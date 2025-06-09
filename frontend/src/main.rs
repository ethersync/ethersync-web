use dioxus::prelude::*;

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

        Counter {}
    }
}


#[component]
fn Counter() -> Element {
    let mut value = use_signal(|| 0);

    rsx! {
        button {
            onclick: move |_| value += 1,
            "counter: {value}"
        }
    }
}
