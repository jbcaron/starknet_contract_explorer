use serde::{Deserialize, Serialize};
use starknet_api::hash::StarkFelt;

use crate::history::History;

#[derive(Serialize, Deserialize, Debug)]
pub struct Contract {
    class_hash: History<StarkFelt>,
    nonce: History<StarkFelt>,
}

impl Contract {
    pub fn new() -> Self {
        Contract {
            class_hash: History::new(),
            nonce: History::new(),
        }
    }

    pub fn push_class_hash(&mut self, index: u64, class_hash: StarkFelt) -> Result<(), ()> {
        self.class_hash.push(index, class_hash)
    }

    pub fn get_class_hash(&self) -> Option<&StarkFelt> {
        self.class_hash.get()
    }

    pub fn get_class_hash_at(&self, index: u64) -> Option<&StarkFelt> {
        self.class_hash.get_at(index)
    }

    pub fn push_nonce(&mut self, index: u64, nonce: StarkFelt) -> Result<(), ()> {
        self.nonce.push(index, nonce)
    }

    pub fn get_nonce(&self) -> Option<&StarkFelt> {
        self.nonce.get()
    }

    pub fn get_nonce_at(&self, index: u64) -> Option<&StarkFelt> {
        self.nonce.get_at(index)
    }

    pub fn revert_to(&mut self, index: u64) {
        self.class_hash.revert_to(index);
        self.nonce.revert_to(index);
    }

    pub fn is_empty(&self) -> bool {
        self.class_hash.is_empty() && self.nonce.is_empty()
    }
}
