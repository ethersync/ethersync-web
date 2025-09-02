use crate::services::automerge_service::AUTOMERGE_ERRORS;
use crate::services::connection_service::AUTOMERGE_MESSAGES;
use dioxus::prelude::*;

#[component]
pub fn AutomergeMessagesView() -> Element {
    let error_messages: Vec<String> = AUTOMERGE_ERRORS
        .iter()
        .map(|error| format!("{error}"))
        .collect();
    let messages = AUTOMERGE_MESSAGES.read().to_owned();
    rsx! {
        section {
            h2 { "Automerge Messages" }

            for text in error_messages {
                p { "{text}" }
            }

            if messages.is_empty() {
                p { "No messages yet!" }
            } else {
                dl {
                    for formatted_message in messages {
                        dt { "{formatted_message.direction} {formatted_message.node_id}" }
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
