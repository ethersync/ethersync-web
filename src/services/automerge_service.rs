use crate::services::connection_service::ConnectionCommand;
use anyhow::{bail, Error, Result};
use automerge::sync::{Message as AutomergeSyncMessage, State as SyncState, State, SyncDoc};
use automerge::{Automerge, ChangeHash, ObjId, ReadDoc};
use dioxus::hooks::use_coroutine_handle;
use dioxus::prelude::{Coroutine, GlobalSignal, Readable, Signal};
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use iroh::NodeId;

#[derive(Clone, PartialEq)]
pub struct AutomergeDocumentFile {
    pub file_name: String,
    pub content: String,
}

pub static AUTOMERGE_ERRORS: GlobalSignal<Vec<Error>> = Signal::global(Vec::new);
pub static FILES: GlobalSignal<Vec<String>> = Signal::global(Vec::new);
pub static SELECTED_FILE: GlobalSignal<Option<AutomergeDocumentFile>> = Signal::global(|| None);

#[derive(Clone)]
pub struct FormattedAutomergeMessage {
    pub direction: String,
    pub node_id: String,
    pub heads: String,
    pub json: String,
}

impl FormattedAutomergeMessage {
    pub fn new(direction: &str, node_id: NodeId, message: &AutomergeSyncMessage) -> Result<Self> {
        let heads = serde_json::to_string_pretty(&message.heads)?;

        let mut message_meta =   HashMap::new();
        message_meta.insert("have".to_string(),  serde_json::to_string_pretty(&message.have)?);
        message_meta.insert("heads".to_string(), heads.clone());
        message_meta.insert("need".to_string(),  serde_json::to_string_pretty(&message.need)?);
        message_meta.insert("version".to_string(), format!("{:?}", message.version));
        let json = serde_json::to_string_pretty(&message_meta)?;
        Ok(Self {
            direction: direction.to_string(),
            node_id: node_id.to_string(),
            heads,
            json,
        })
    }
}

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
            apply_message(doc, state, message).await?;

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
                connection_service.send(ConnectionCommand::SendMessage { message });
            }
        }
    }
    Ok(())
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
            AUTOMERGE_ERRORS.write().push(error);
        }
    }
}
