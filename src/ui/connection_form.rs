use crate::services::node_service::{NodeCommand, SecretAddress};
use dioxus::prelude::*;
use std::string::ToString;

#[component]
fn SimpleForm() -> Element {
    rsx! {
        fieldset {
            label {
                for: "join_code",
                "magic wormhole code:"
            }

            input {
                id: "join_code",
                name: "join_code",
                style: "min-width: 40em;"
            }
        }
    }
}

#[component]
fn AdvancedForm() -> Element {
    rsx! {
        fieldset {
            label {
                for: "peer_node_id",
                "peer node id:"
            }

            input {
                id: "peer_node_id",
                name: "peer_node_id",
                style: "min-width: 40em;"
            }
        }

        fieldset {
            label {
                for: "peer_passphrase",
                "peer passphrase:"
            }

            // TODO: should this be type: password?
            input {
                id: "peer_passphrase",
                name: "peer_passphrase",
                style: "min-width: 40em;"
            }
        }
    }
}

#[component]
pub fn ConnectionForm() -> Element {
    let node_service = use_coroutine_handle::<NodeCommand>();

    let mut form_error = use_signal(|| "".to_string());
    let mut mode = use_signal(|| "simple".to_string());

    let onsubmit = move |event: FormEvent| {
        event.stop_propagation();
        let form_data = event.values();
        form_error.set("".to_string());

        match mode.read().as_str() {
            "simple" => {
                let join_code = form_data["join_code"].as_value();
                node_service.send(NodeCommand::ConnectByJoinCode { join_code });
            }
            "advanced" => {
                match SecretAddress::from_string(
                    form_data["peer_node_id"].as_value(),
                    form_data["peer_passphrase"].as_value(),
                ) {
                    Ok(secret_address) => node_service.send(NodeCommand::ConnectByAddress {
                        secret_address: Box::new(secret_address),
                    }),
                    Err(error) => {
                        form_error.set(format!("{error}"));
                    }
                }
            }
            _ => {
                panic!("Unexpected form mode: {mode}")
            }
        };
    };

    rsx! {
        section {
            h2 { "Bidirectional Connection" }

            "{form_error}"

            form {
                onsubmit,

                fieldset {
                    for value in ["simple", "advanced"] {
                        label {
                            input {
                                type: "radio",
                                name: "mode",
                                checked: *mode.read() == value,
                                oninput: move |_| mode.set(value.to_string()),
                                value: value
                            },
                            "{value}"
                        }
                    }
                }

                match mode.read().as_str() {
                    "simple" => rsx! {
                        SimpleForm { }
                    },
                    "advanced" => rsx! {
                        AdvancedForm { }
                    },
                    _ => {
                        panic!("Unexpected form mode: {mode}")
                    }
                }

                button {
                    type: "submit",
                    "connect"
                }
            }
        }
    }
}
