use crate::services::automerge_service::AUTOMERGE_EVENTS;
use crate::ui::file_list::FileList;
use dioxus::prelude::*;

#[component]
pub fn AutomergeDocumentView() -> Element {
    rsx! {
        section {
            h2 { "Automerge Document" }

            FileList {}

            hr { }

            ul {
                for event in AUTOMERGE_EVENTS.iter() {
                    li {
                        for line in event.to_string().split('\n') {
                            "{line}" br {}
                        }
                    }
                }
            }
        }
    }
}
