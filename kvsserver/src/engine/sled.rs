use super::{KvsEngine, Result, KvsError};
use sled::Db;
use std::path::PathBuf;

pub struct SledKvStore {
    tree : Db,
}

impl SledKvStore {
    pub fn new<P : Into<PathBuf>>(path : P) -> Result<Self> {
        let tree = Db::open(&path.into())?;
        tree.flush()?;
        Ok(SledKvStore {
            tree
        })
    }
}

impl KvsEngine for SledKvStore {
    fn get(&mut self, key : String) -> Result<Option<String>> {
        match self.tree.get(key)? {
            Some(value) => Ok(Some(std::str::from_utf8(value.as_ref())?.to_string())),
            None => Ok(None),
        }
    }

    fn set(&mut self, key : String, value : String) -> Result<()> {
        self.tree.insert(key.into_bytes(), value.into_bytes())?;
        self.tree.flush()?;
        Ok(())
    }

    fn remove(&mut self, key : String) -> Result<()> {
        if let None = self.tree.remove(key)? {
            return Err(KvsError::KeyNotFound);
        }
        self.tree.flush()?;
        Ok(())
    }
}
