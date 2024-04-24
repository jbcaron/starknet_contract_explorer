use serde::Deserialize;
use starknet_api::hash::StarkFelt;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct StateUpdate {
    pub block_hash: StarkFelt,
    pub new_root: StarkFelt,
    pub old_root: StarkFelt,
    pub state_diff: StateDiff,
}

#[derive(Deserialize, Debug)]
pub struct StateDiff {
    pub storage_diffs: HashMap<StarkFelt, Vec<StorageDiff>>,
    pub deployed_contracts: Vec<DeployedContract>,
    pub old_declared_contracts: Vec<StarkFelt>,
    pub declared_classes: Vec<DeclaredClass>,
    pub nonces: HashMap<StarkFelt, StarkFelt>,
    pub replaced_classes: Vec<DeployedContract>,
}

#[derive(Deserialize, Debug)]
pub struct StorageDiff {
    pub key: StarkFelt,
    pub value: StarkFelt,
}

#[derive(Deserialize, Debug)]
pub struct DeployedContract {
    pub address: StarkFelt,
    pub class_hash: StarkFelt,
}

#[derive(Deserialize, Debug)]
pub struct DeclaredClass {
    pub class_hash: StarkFelt,
    pub compiled_class_hash: StarkFelt,
}

#[derive(Deserialize, Debug)]
pub struct ContractClass {
    pub address: StarkFelt,
    pub class_hash: StarkFelt,
}
