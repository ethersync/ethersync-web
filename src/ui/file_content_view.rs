use crate::shared::automerge_document::AutomergeDocumentFile;
use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct FileContentViewProps {
    selected_file: AutomergeDocumentFile,
}

#[component]
pub fn FileContentView(props: FileContentViewProps) -> Element {
    let file_name = props.selected_file.file_name;
    let content = props.selected_file.content.unwrap_or_default();
    rsx! {
        section {
            h2 {
                "File Content",
                code { "({file_name})" }
            }

            textarea {
                disabled: true,
                rows: 10,
                "{content}"
            }
        }
    }
}
