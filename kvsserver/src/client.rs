use std::net::{TcpStream, TcpListener, ToSocketAddrs};
use serde::Deserialize;
use serde_json::{self, Serializer};
use serde_json::de::{Deserializer, IoRead};
use crate::errors::{Result, KvsError};
use crate::common::{Request, GetResponse, SetResponse, RemoveResponse};
use std::io::{BufReader, BufWriter, Write};

const RETRY_TIMES : u64 = 100;

/// Use buffered TcpStream with a Deserializer to get the response
/// from remote server, and Buffered Writer of TcpStreamto send request
pub struct KvsClient {
    reader : Deserializer<IoRead<BufReader<TcpStream>>>,
    writer : BufWriter<TcpStream>,
    connect_times : u64,
}


impl KvsClient {
   /// create new KvsClient and connect to the remote addresss
   pub fn new<A : ToSocketAddrs>(addr : A) -> Result<KvsClient> {
        let mut stream = TcpStream::connect(addr)?;

        let writer = BufWriter::new(stream.try_clone()?);
        let reader = Deserializer::new(IoRead::new(BufReader::new(stream)));
        Ok(KvsClient {
           reader,
           writer,
           connect_times : 10,
        })
    }
    /// set the key-value pair to the KvStore Engine
    /// Ok(()) => set the value and receive response successful
    /// Err(err) => Some error occured
    pub fn set(&mut self, key : String, value : String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Set(key, value))?;
        // ensure the request send successful
        self.writer.flush()?;

        let response = SetResponse::deserialize(&mut self.reader)?;
        match response {
            SetResponse::Ok(()) => Ok(()),
            SetResponse::Err(err) => Err(KvsError::StringError(err)),
        }
    }


    /// get the value use key from KvStore Engine
    /// Ok(Some(value)) => get value successful
    /// Ok(None)  => Key not found  
    pub fn get(&mut self, key : String)  -> Result<Option<String>> {
        serde_json::to_writer(&mut self.writer, &Request::Get(key))?;
        self.writer.flush()?;


        let resp = GetResponse::deserialize(&mut self.reader)?;

        match resp {
            GetResponse::Ok(value) => Ok(value),
            GetResponse::Err(err) => Err(KvsError::StringError(err)),
        }

    }

    /// remove the key from the KvStore Engine
    /// Ok(()) => remove item correct and receive the response
    /// Err(err) => Err occured
    pub fn remove(&mut self, key : String) -> Result<()> {
        serde_json::to_writer(&mut self.writer, &Request::Remove(key))?;
        self.writer.flush()?;

        let response = RemoveResponse::deserialize(&mut self.reader)?;
        match response {
            RemoveResponse::Ok(()) => Ok(()),
            RemoveResponse::Err(err) => Err(KvsError::StringError(err)),
        }
    }
}
