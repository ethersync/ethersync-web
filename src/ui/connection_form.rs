use crate::shared::secret_address::{get_secret_address_from_wormhole, SecretAddress};
use dioxus::prelude::*;
use std::string::ToString;

#[derive(PartialEq, Props, Clone)]
pub struct ConnectionFormProps {
    connect_to_peer: Callback<SecretAddress>,
}

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
pub fn ConnectionForm(props: ConnectionFormProps) -> Element {
    let mut mode = use_signal(|| "simple".to_string());

    let onsubmit = move |event: FormEvent| async move {
        event.stop_propagation();
        let form_data = event.values();

        let secret_address = match mode.read().as_str() {
            "simple" => {
                let code = form_data["join_code"].as_value();
                get_secret_address_from_wormhole(&code).await
            }
            "advanced" => SecretAddress::from_string(
                form_data["peer_node_id"].as_value(),
                form_data["peer_passphrase"].as_value(),
            ),
            _ => {
                panic!("Unexpected form mode: {mode}")
            }
        };

        props
            .connect_to_peer
            .call(secret_address.expect("Invalid secret address!"));
    };

    rsx! {
        section {
            h2 { "Bidirectional Connection" }

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
