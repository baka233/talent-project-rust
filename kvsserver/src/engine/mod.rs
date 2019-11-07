use super::errors::*;

pub trait KvsEngine { 
    fn set(&mut self, key : String, value : String) -> Result<()>;

    fn get(&mut self, key : String)  -> Result<Option<String>>;

    fn remove(key : String) -> Result<()>;
}

mod kv;

pub use self::kv::KvStore;
