use failure::Fail;
use std::io;
use sled;

///Error type for kvs
#[derive(Fail, Debug)]
pub enum KvsError {
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),
    #[fail(display = "{}", _0)]
    Sled(#[cause] sled::Error),
    #[fail(display = "{}", _0)]
    Utf8Error(#[cause] std::str::Utf8Error),
    #[fail(display = "Key not found")]
    KeyNotFound,
    #[fail(display = "{}", _0)]
    StringError(String),
    #[fail(display = "Unexpected command type")]
    UnexpectedCommandType,
}

impl From<io::Error> for KvsError {
    fn from(err : io::Error) -> KvsError {
        KvsError::Io(err)
    }
}


impl From<serde_json::Error> for KvsError {
    fn from(err : serde_json::Error) -> KvsError {
        KvsError::Serde(err)
    }
}

impl From<sled::Error> for KvsError {
    fn from(err : sled::Error)  -> KvsError {
        KvsError::Sled(err)
    }    
}

impl From<std::str::Utf8Error> for KvsError {
    fn from(err : std::str::Utf8Error)  -> KvsError {
        KvsError::Utf8Error(err)
    }    
}


pub type Result<T> = std::result::Result<T, KvsError>;
