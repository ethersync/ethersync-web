use automerge::sync::{Message as AutomergeSyncMessage, State as SyncState, SyncDoc};
use automerge::{Automerge, ReadDoc};
use std::cell::RefCell;

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

    fn top_level_map_obj(&self, name: &str) -> Option<automerge::ObjId> {
        self.doc
            .borrow()
            .get(automerge::ROOT, name)
            .expect(&format!("{name} not found!"))
            .map(|r| r.1)
    }

    pub fn files(&self) -> Vec<String> {
        if let Some(file_map) = self.top_level_map_obj("files") {
            self.doc.borrow().keys(file_map).collect()
        } else {
            vec![]
        }
    }
}
