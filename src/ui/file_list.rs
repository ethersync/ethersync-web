use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct FileListProps {
    files: Vec<String>,
}

#[component]
pub fn FileList(props: FileListProps) -> Element {
    rsx! {
        section {
            h2 { "Files" }

            ul {
                if props.files.is_empty() {
                    li { "No files!" }
                } else {
                    for name in props.files {
                        li { "{name}" }
                    }
                }
            }
        }
    }
}
