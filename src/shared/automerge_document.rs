use automerge::sync::{Message as AutomergeSyncMessage, State as SyncState, SyncDoc};
use automerge::{Automerge, ObjId, ReadDoc};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::{SinkExt, StreamExt};
use std::cell::RefCell;
use tokio::sync::broadcast::Sender;

pub struct AutomergeDocument {
    doc: RefCell<Automerge>,
    state: RefCell<SyncState>,
}

impl Default for AutomergeDocument {
    fn default() -> Self {
        Self {
            doc: RefCell::new(Automerge::new()),
            state: RefCell::new(SyncState::new()),
        }
    }
}

pub struct FormattedAutomergeMessage {
    pub direction: String,
    pub heads: String,
    pub json: String,
}

impl FormattedAutomergeMessage {
    pub fn new(direction: &str, message: &AutomergeSyncMessage) -> Self {
        let json = serde_json::to_string_pretty(&message).expect("Converting to JSON failed!");
        Self {
            direction: direction.to_string(),
            heads: message
                .heads
                .iter()
                .map(|h| h.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            json,
        }
    }
}

impl AutomergeDocument {
    pub async fn create_message(&self) -> Option<AutomergeSyncMessage> {
        let state_ref = &mut *self.state.borrow_mut();
        self.doc.borrow().generate_sync_message(state_ref)
    }

    pub async fn apply_message(&self, message: AutomergeSyncMessage) -> Self {
        let mut new_doc = self.doc.borrow().fork();
        let state_ref = &mut *self.state.borrow_mut();
        new_doc
            .receive_sync_message(state_ref, message)
            .expect("Failed to apply message!");
        self.doc
            .borrow_mut()
            .merge(&mut new_doc)
            .expect("Failed to merge doc!");
        Self {
            doc: RefCell::new(new_doc),
            state: RefCell::new(state_ref.clone()),
        }
    }

    fn object_id_by_name(&self, parent: ObjId, name: &str) -> Option<ObjId> {
        self.doc
            .borrow()
            .get(parent, name)
            .expect(&format!("{name} not found!"))
            .map(|entry| entry.1)
    }

    fn files_object(&self) -> Option<ObjId> {
        self.object_id_by_name(automerge::ROOT, "files")
    }

    pub fn files(&self) -> Vec<String> {
        self.files_object().map_or(vec![], |object_id| {
            self.doc.borrow().keys(object_id).collect()
        })
    }

    pub fn file_content(&self, file_name: String) -> Option<String> {
        self.files_object()
            .and_then(|parent_id| self.object_id_by_name(parent_id, &file_name))
            .and_then(|file_id| self.doc.borrow().text(file_id).ok())
    }
}

pub enum AutomergeDocumentAction {
    Apply {
        message: AutomergeSyncMessage,
    },
    SelectFile {
        file_name: String,
    },
    Sync {
        outgoing_message_tx: Sender<AutomergeSyncMessage>,
    },
}

#[derive(Clone, PartialEq)]
pub struct AutomergeDocumentFile {
    pub file_name: String,
    pub content: Option<String>,
}

pub enum AutomergeDocumentEvent {
    Changed {
        files: Vec<String>,
        selected_file: Option<AutomergeDocumentFile>,
    },
}

pub struct AutomergeDocumentHandler {
    pub doc_event_tx: UnboundedSender<AutomergeDocumentEvent>,
}

impl AutomergeDocumentHandler {
    pub async fn handle_actions(
        &mut self,
        mut action_rx: UnboundedReceiver<AutomergeDocumentAction>,
    ) {
        // TODO: load content from local storage?
        let mut doc = AutomergeDocument::default();

        let mut selected_file = None;

        while let Some(action) = action_rx.next().await {
            match action {
                AutomergeDocumentAction::Apply { message } => {
                    doc = doc.apply_message(message).await;
                    self.send_change_event(&doc, &selected_file).await;
                }
                AutomergeDocumentAction::SelectFile { ref file_name } => {
                    selected_file = Some(AutomergeDocumentFile {
                        file_name: file_name.clone(),
                        content: doc.file_content(file_name.clone()),
                    });
                    self.send_change_event(&doc, &selected_file).await;
                }
                AutomergeDocumentAction::Sync {
                    outgoing_message_tx,
                } => {
                    if let Some(message) = doc.create_message().await {
                        // TODO: proper error handling
                        outgoing_message_tx
                            .send(message)
                            .expect("failed to send sync!");
                    }
                }
            }
        }
    }

    async fn send_change_event(
        &mut self,
        doc: &AutomergeDocument,
        selected_file: &Option<AutomergeDocumentFile>,
    ) {
        // TODO: proper error handling
        self.doc_event_tx
            .send(AutomergeDocumentEvent::Changed {
                files: doc.files(),
                selected_file: selected_file.clone(),
            })
            .await
            .expect("failed to send event!");
    }
}
