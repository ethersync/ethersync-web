use crate::services::connection_service::ConnectionCommand;
use anyhow::{bail, Error, Result};
use automerge::sync::{Message as AutomergeSyncMessage, State as SyncState, State, SyncDoc};
use automerge::{Automerge, ChangeHash, ObjId, ReadDoc};
use chrono::{DateTime, Local};
use dioxus::hooks::use_coroutine_handle;
use dioxus::prelude::{Coroutine, GlobalSignal, Readable, Signal};
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use std::fmt::Display;

#[derive(Clone, PartialEq)]
pub struct AutomergeDocumentFile {
    pub file_name: String,
    pub content: String,
}

pub struct MessageDetails {
    heads: String,
    last_sync: String,
    need: String,
    version: String,
}

impl MessageDetails {
    fn from_message(message: &AutomergeSyncMessage) -> Result<Self> {
        let last_sync: Vec<ChangeHash> = message
            .have
            .iter()
            .flat_map(|h| h.last_sync.clone())
            .collect();
        Ok(Self {
            last_sync: serde_json::to_string_pretty(&last_sync)?,
            heads: serde_json::to_string_pretty(&message.heads)?,
            need: serde_json::to_string_pretty(&message.need)?,
            version: format!("{:?}", message.version),
        })
    }
}

impl Display for MessageDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "last_sync: {}\nheads: {}\nneed: {}\nversion: {}",
            self.last_sync, self.heads, self.need, self.version
        )
    }
}

pub enum AutomergeEvent {
    AppliedSyncMessage {
        date_time: DateTime<Local>,
        details: MessageDetails,
    },
    CreatedSyncMessage {
        date_time: DateTime<Local>,
        details: MessageDetails,
    },
    Error {
        date_time: DateTime<Local>,
        error: Error,
    },
}

impl Display for AutomergeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutomergeEvent::AppliedSyncMessage { date_time, details } => {
                write!(f, "{date_time}: applied sync message:\n{details}")
            }
            AutomergeEvent::CreatedSyncMessage { date_time, details } => {
                write!(f, "{date_time}: created sync message:\n{details}")
            }
            AutomergeEvent::Error { date_time, error } => {
                write!(f, "{date_time}: automerge error {error}")
            }
        }
    }
}

pub static AUTOMERGE_EVENTS: GlobalSignal<Vec<AutomergeEvent>> = Signal::global(Vec::new);

pub static FILES: GlobalSignal<Vec<String>> = Signal::global(Vec::new);
pub static SELECTED_FILE: GlobalSignal<Option<AutomergeDocumentFile>> = Signal::global(|| None);

async fn apply_message(
    doc: &mut Automerge,
    state: &mut State,
    message: AutomergeSyncMessage,
) -> Result<Vec<ChangeHash>> {
    let mut new_doc = doc.fork();
    new_doc.receive_sync_message(state, message)?;
    Ok(doc.merge(&mut new_doc)?)
}

fn object_id_by_name(doc: &Automerge, parent: ObjId, name: &str) -> Result<ObjId> {
    if let Some(object_id) = doc.get(parent, name)?.map(|entry| entry.1) {
        return Ok(object_id);
    }

    bail!("no object '{name}' found!")
}

fn files_object(doc: &Automerge) -> Result<ObjId> {
    object_id_by_name(doc, automerge::ROOT, "files")
}

fn files(doc: &Automerge) -> Result<Vec<String>> {
    Ok(doc.keys(files_object(doc)?).collect())
}

fn file_content(doc: &Automerge, file_name: &str) -> Result<String> {
    let object_id = object_id_by_name(doc, files_object(doc)?, file_name)?;
    Ok(doc.text(object_id)?)
}

fn select_file(doc: &Automerge, file_name: &str) -> Result<()> {
    *SELECTED_FILE.write() = Some(AutomergeDocumentFile {
        file_name: file_name.to_owned(),
        content: file_content(doc, file_name)?,
    });
    Ok(())
}

pub enum AutomergeCommand {
    ApplyMessage { message: AutomergeSyncMessage },
    SelectFile { file_name: String },
    StartSync,
}

async fn handle_automerge_command(
    doc: &mut Automerge,
    state: &mut State,
    command: AutomergeCommand,
    connection_service: Coroutine<ConnectionCommand>,
) -> Result<()> {
    match command {
        AutomergeCommand::ApplyMessage { message } => {
            let details = MessageDetails::from_message(&message)?;
            apply_message(doc, state, message).await?;
            AUTOMERGE_EVENTS
                .write()
                .push(AutomergeEvent::AppliedSyncMessage {
                    date_time: Local::now(),
                    details,
                });

            *FILES.write() = files(doc)?;

            if let Some(previous_selected_file) = SELECTED_FILE.read().as_ref() {
                let file_name = &previous_selected_file.file_name;
                if FILES.read().contains(file_name) {
                    select_file(doc, file_name)?;
                } else {
                    *SELECTED_FILE.write() = None;
                }
            }
        }
        AutomergeCommand::SelectFile { ref file_name } => {
            select_file(doc, file_name)?;
        }
        AutomergeCommand::StartSync => {
            while let Some(message) = doc.generate_sync_message(state) {
                let details = MessageDetails::from_message(&message)?;
                AUTOMERGE_EVENTS
                    .write()
                    .push(AutomergeEvent::CreatedSyncMessage {
                        date_time: Local::now(),
                        details,
                    });
                connection_service.send(ConnectionCommand::SendMessage { message });
            }
        }
    }
    Ok(())
}

fn handle_error(error: Error) {
    AUTOMERGE_EVENTS.write().push(AutomergeEvent::Error {
        date_time: Local::now(),
        error,
    });
}

pub async fn start_automerge_service(mut commands_rx: UnboundedReceiver<AutomergeCommand>) {
    let connection_service = use_coroutine_handle::<ConnectionCommand>();

    // TODO: load content from local storage?
    let mut doc = Automerge::default();
    let mut state = SyncState::default();

    while let Some(command) = commands_rx.next().await {
        if let Err(error) =
            handle_automerge_command(&mut doc, &mut state, command, connection_service).await
        {
            handle_error(error);
        }
    }
}
