use crate::services::automerge_service::{AutomergeCommand, FILES};
use dioxus::prelude::*;

#[component]
pub fn FileList() -> Element {
    let files = FILES.read().to_owned();
    let automerge_service = use_coroutine_handle::<AutomergeCommand>();

    rsx! {
        section {
            h2 { "Files" }

            ul {
                if files.is_empty() {
                    li { "No files!" }
                } else {
                    for file_name in files {
                        li {
                            a {
                                // TODO: use real href and router to allow permalinks
                                href: "#",
                                onclick: move |_| {
                                    automerge_service.send(AutomergeCommand::SelectFile {
                                        file_name: file_name.clone()
                                    });
                                },
                                "{file_name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
