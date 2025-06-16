use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct IncomingMessagesViewProps {
    incoming_messages: Signal<Vec<String>>,
}

#[component]
pub fn IncomingMessagesView(props: IncomingMessagesViewProps) -> Element {
    rsx! {
        section {
            h2 { "Incoming Messages" }

            ul {
                if (&*props.incoming_messages.read()).is_empty() {
                    li { "No messages yet!" }
                } else {
                    for text in &*props.incoming_messages.read() {
                        li { "{text}" }
                    }
                }
            }
        }
    }
}
