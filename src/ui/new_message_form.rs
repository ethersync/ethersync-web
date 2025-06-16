use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct NewMessageFormProps {
    send_message: Callback<String>,
}

#[component]
pub fn NewMessageForm(props: NewMessageFormProps) -> Element {
    let mut message_text = use_signal(|| "".to_string());
    let mut peer_node_id = use_signal(|| "".to_string());

    rsx! {
        section {
            h2 { "New Message" }

            fieldset {
                label {
                    for: "message_text",
                    "text:"
                }

                input {
                    id: "message_text",
                    value: "{message_text}",
                    oninput: move |event| message_text.set(event.value().clone()),
                    style: "min-width: 40em;"
                }
            }

            button {
                disabled: (&*message_text.read()).is_empty(),
                onclick: move |_| async move {
                    let message =(&*message_text.read()).clone();
                    props.send_message.call(message);

                    // TODO: await before resetting input
                    message_text.set("".to_string());
                },
                "send"
            }
        }
    }
}
