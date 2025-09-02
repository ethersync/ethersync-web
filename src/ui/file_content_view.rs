use crate::services::automerge_service::SELECTED_FILE;
use dioxus::prelude::*;

#[component]
pub fn FileContentView() -> Element {
    if let Some(selected_file) = SELECTED_FILE.read().as_ref() {
        rsx! {
            section {
                h2 {
                    "File Content",
                    code { "({selected_file.file_name})" }
                }

                textarea {
                    disabled: true,
                    rows: 10,
                    "{selected_file.content}"
                }
            }
        }
    } else {
        rsx! {}
    }
}
