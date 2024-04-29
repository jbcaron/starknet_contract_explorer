use rocksdb::{ColumnFamilyDescriptor, DBCompressionType, Options, DB};
use starknet_api::hash::StarkFelt;

use crate::contract::Contract;
use crate::history::History;

pub struct Database {
    db: DB,
    path: String,
}

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
    #[error("Decode error")]
    DecodeError,
    #[error("Encode error")]
    EncodeError,
    #[error("History error")]
    HistoryError,
    #[error("Iterator error")]
    IteratorError,
    #[error("RocksDB error: {0}")]
    RocksDBError(rocksdb::Error),
}

impl Database {
    pub fn new(path: &str) -> Result<Self, DatabaseError> {
        let mut db_opts = Options::default();
        db_opts.set_compression_type(DBCompressionType::Zstd);
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let cf_opts = Options::default();
        let cf1 = ColumnFamilyDescriptor::new("contract", cf_opts.clone());
        let cf2 = ColumnFamilyDescriptor::new("key", cf_opts);

        let db = DB::open_cf_descriptors(&db_opts, path, vec![cf1, cf2])
            .map_err(|e| DatabaseError::RocksDBError(e))?;

        Ok(Database {
            db,
            path: path.to_string(),
        })
    }

    fn insert(&self, cf: &str, key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        let cf = self
            .db
            .cf_handle(cf)
            .ok_or(DatabaseError::ColumnNotFound(cf.to_string()))?;

        self.db
            .put_cf(cf, key, value)
            .map_err(|e| DatabaseError::RocksDBError(e))
    }

    fn get(&self, cf: &str, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        let cf = self
            .db
            .cf_handle(cf)
            .ok_or(DatabaseError::ColumnNotFound(cf.to_string()))?;

        self.db
            .get_cf(cf, key)
            .map_err(|e| DatabaseError::RocksDBError(e))
    }

    fn delete(&self, cf: &str, key: &[u8]) -> Result<(), DatabaseError> {
        let cf = self
            .db
            .cf_handle(cf)
            .ok_or(DatabaseError::ColumnNotFound(cf.to_string()))?;

        self.db
            .delete_cf(cf, key)
            .map_err(|e| DatabaseError::RocksDBError(e))
    }

    fn iter(&self, cf: &str) -> Result<rocksdb::DBIterator, DatabaseError> {
        let cf = self
            .db
            .cf_handle(cf)
            .ok_or(DatabaseError::ColumnNotFound(cf.to_string()))?;

        let mode = rocksdb::IteratorMode::Start;

        Ok(self.db.iterator_cf(cf, mode))
    }

    pub fn destroy(&self) -> Result<(), DatabaseError> {
        DB::destroy(&Options::default(), self.path.as_str())
            .map_err(|e| DatabaseError::RocksDBError(e))
    }

    pub fn repair(&self) -> Result<(), DatabaseError> {
        DB::repair(&Options::default(), self.path.as_str())
            .map_err(|e| DatabaseError::RocksDBError(e))
    }

    pub fn flush(&self) -> Result<(), DatabaseError> {
        self.db.flush().map_err(|e| DatabaseError::RocksDBError(e))
    }

    pub fn insert_key(
        &self,
        contract: StarkFelt,
        key: StarkFelt,
        value: StarkFelt,
        index: u64,
    ) -> Result<(), DatabaseError> {
        let mut db_key = Vec::new();
        db_key.extend_from_slice(contract.bytes());
        db_key.extend_from_slice(key.bytes());

        let encoded = self.get("key", &db_key)?;
        let mut history: History<StarkFelt> = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => History::new(),
        };

        history
            .push(index, value)
            .map_err(|_| DatabaseError::HistoryError)?;
        let encoded = bincode::serialize(&history).map_err(|_| DatabaseError::EncodeError)?;

        self.insert("key", &db_key, &encoded)
    }

    pub fn get_key(
        &self,
        contract: StarkFelt,
        key: StarkFelt,
    ) -> Result<Option<StarkFelt>, DatabaseError> {
        let mut db_key = Vec::new();
        db_key.extend_from_slice(contract.bytes());
        db_key.extend_from_slice(key.bytes());

        let encoded = self.get("key", &db_key)?;
        let history: History<StarkFelt> = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => return Ok(None),
        };

        Ok(history.get().cloned())
    }

    pub fn get_key_at(
        &self,
        contract: StarkFelt,
        key: StarkFelt,
        index: u64,
    ) -> Result<Option<StarkFelt>, DatabaseError> {
        let mut db_key = Vec::new();
        db_key.extend_from_slice(contract.bytes());
        db_key.extend_from_slice(key.bytes());

        let encoded = self.get("key", &db_key)?;
        let history: History<StarkFelt> = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => return Ok(None),
        };

        Ok(history.get_at(index).cloned())
    }

    pub fn insert_nonce(
        &self,
        contract: StarkFelt,
        nonce: StarkFelt,
        index: u64,
    ) -> Result<(), DatabaseError> {
        let db_key = contract.bytes();

        let encoded = self.get("contract", db_key)?;
        let mut contract: Contract = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => Contract::new(),
        };

        contract
            .push_nonce(index, nonce)
            .map_err(|_| DatabaseError::HistoryError)?;

        let encoded = bincode::serialize(&contract).map_err(|_| DatabaseError::EncodeError)?;

        self.insert("contract", db_key, &encoded)
    }

    pub fn get_nonce(&self, contract: StarkFelt) -> Result<Option<StarkFelt>, DatabaseError> {
        let db_key = contract.bytes();

        let encoded = self.get("contract", db_key)?;
        let contract: Contract = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => return Ok(None),
        };

        Ok(contract.get_nonce().cloned())
    }

    pub fn get_nonce_at(
        &self,
        contract: StarkFelt,
        index: u64,
    ) -> Result<Option<StarkFelt>, DatabaseError> {
        let db_key = contract.bytes();

        let encoded = self.get("contract", db_key)?;
        let contract: Contract = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => return Ok(None),
        };

        Ok(contract.get_nonce_at(index).cloned())
    }

    pub fn insert_class_hash(
        &self,
        contract: StarkFelt,
        class_hash: StarkFelt,
        index: u64,
    ) -> Result<(), DatabaseError> {
        let db_key = contract.bytes();

        let encoded = self.get("contract", db_key)?;
        let mut contract: Contract = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => Contract::new(),
        };

        contract
            .push_class_hash(index, class_hash)
            .map_err(|_| DatabaseError::HistoryError)?;

        let encoded = bincode::serialize(&contract).map_err(|_| DatabaseError::EncodeError)?;

        self.insert("contract", db_key, &encoded)
    }

    pub fn get_class_hash(&self, contract: StarkFelt) -> Result<Option<StarkFelt>, DatabaseError> {
        let db_key = contract.bytes();

        let encoded = self.get("contract", db_key)?;
        let contract: Contract = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => return Ok(None),
        };

        Ok(contract.get_class_hash().cloned())
    }

    pub fn get_class_hash_at(
        &self,
        contract: StarkFelt,
        index: u64,
    ) -> Result<Option<StarkFelt>, DatabaseError> {
        let db_key = contract.bytes();

        let encoded = self.get("contract", db_key)?;
        let contract: Contract = match encoded {
            Some(encoded) => {
                bincode::deserialize(&encoded).map_err(|_| DatabaseError::DecodeError)?
            }
            None => return Ok(None),
        };

        Ok(contract.get_class_hash_at(index).cloned())
    }

    pub fn revert_to(&self, index: u64) -> Result<(), DatabaseError> {
        let cf_handle = self
            .db
            .cf_handle("key")
            .ok_or(DatabaseError::ColumnNotFound("key".to_string()))?;
        let iter = self.db.iterator_cf(cf_handle, rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, encoded) = item.map_err(|_| DatabaseError::IteratorError)?;

            let mut history: History<StarkFelt> =
                bincode::deserialize(&*encoded).map_err(|_| DatabaseError::DecodeError)?;

            history.revert_to(index);
            if history.is_empty() {
                self.delete("key", &*key)?;
            } else {
                let encoded =
                    bincode::serialize(&history).map_err(|_| DatabaseError::EncodeError)?;
                self.insert("key", &*key, &encoded)?;
            }
        }

        let cf_handle = self
            .db
            .cf_handle("contract")
            .ok_or(DatabaseError::ColumnNotFound("contract".to_string()))?;
        let iter = self.db.iterator_cf(cf_handle, rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, encoded) = item.map_err(|_| DatabaseError::IteratorError)?;

            let mut contract: Contract =
                bincode::deserialize(&*encoded).map_err(|_| DatabaseError::DecodeError)?;

            contract.revert_to(index);
            if contract.is_empty() {
                self.delete("contract", &*key)?;
            } else {
                let encoded =
                    bincode::serialize(&contract).map_err(|_| DatabaseError::EncodeError)?;
                self.insert("contract", &*key, &encoded)?;
            }
        }

        Ok(())
    }
}
