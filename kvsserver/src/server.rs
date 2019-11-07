use std::net::{TcpStream, TcpListener, ToSocketAddrs};
use serde_json::{self, Deserializer, Serializer};
use crate::errors::{Result, KvsError};
use crate::common::*;
use crate::engine::{KvsEngine, KvStore};


/// The server of the KvStroe
pub struct KvServer<E : KvsEngine> {
    engine : E, 
}

impl<E : KvsEngine> KvServer<E> {
    /// Create new KvServer use specified engine
    pub fn new(engine : E) -> Self {
        KvServer {
            engine 
        }  
    }

    /// Start server and handle request, now we only support single request!
    pub fn run<A : ToSocketAddrs>(&mut self, addr : A) -> Result<()>{
        let mut listener = TcpListener::bind(addr)?;

        for stream in listener.incoming() {
            if let Err(err) = self.handle_request(stream?) {
                debug!("Error {:?} has occured in handling request", err); 
            }
        }

        Ok(())
    }

    /// handler of kvserver
    pub fn handle_request(&mut self, streamer : TcpStream) -> Result<()> {
        let client_addr = streamer.peer_addr()?; 
        let req_reader = Deserializer::from_reader(streamer).into_iter::<Request>();

        macro_rules! send_response {
            ($resp : expr)  => { {
                let resq = $resp;
                serde_json::to_writer(streamer, $resp)?;
                streamer.flush();
                debug!("streamer send to {}, {:?}", client_addr, $resp);
            };};
        }

        for request in req_reader {
            let request = request?;
            match request {
                Request::Get(key) => send_response!(match KvsEngine::get(&mut self.engine, key) {
                    Ok(value) => GetResponse::Ok(value),
                    Err(err)  => GetResponse::Err(format!("{}", err)) 
                }),
                Request::Set(key, value) => send_response!(match self.engine.set(key, value) {
                    Ok(()) => SetResponse::Ok(()),
                    Err(err) => SetResponse::Err(format!("{}", err))
                }),
                Request::Remove(key) => send_response!(match self.engine.remove(key) {
                    Ok(()) => RemoveResponse::Ok(()) ,
                    Err(err) => RemoveResponse::Err(format!("{}", err))
                }),
            };
        }
        
        Ok(())
    }
}

