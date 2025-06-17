use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct FileContentViewProps {
    content: String,
    file_name: String,
}

#[component]
pub fn FileContentView(props: FileContentViewProps) -> Element {
    rsx! {
        section {
            h2 {
                "File Content",
                if !props.file_name.is_empty() {
                    code { "({props.file_name})" }
                }
            }

            textarea {
                disabled: true,
                rows: 10,
                if props.file_name.is_empty() {
                    "no file selected!"
                } else {
                    "{props.content}"
                }
            }
        }
    }
}
