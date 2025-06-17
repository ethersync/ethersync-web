use dioxus::prelude::*;
use crate::shared::automerge_document::FormattedAutomergeMessage;

#[derive(PartialEq, Props, Clone)]
pub struct AutomergeMessagesViewProps {
    automerge_messages: Signal<Vec<FormattedAutomergeMessage>>,
}

#[component]
pub fn AutomergeMessagesView(props: AutomergeMessagesViewProps) -> Element {
    rsx! {
        section {
            h2 { "Automerge Messages" }

            if (&*props.automerge_messages.read()).is_empty() {
                p { "No messages yet!" }
            } else {
                dl {
                    for formatted_message in &*props.automerge_messages.read() {
                        dt { "{formatted_message.direction}" }
                        dd {
                            details {
                                summary {
                                    if formatted_message.heads.is_empty() {
                                        "no heads"
                                    } else {
                                        "{formatted_message.heads}"
                                    }
                                }
                                pre { "{formatted_message.json}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
