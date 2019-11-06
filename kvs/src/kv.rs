use std::collections::{HashMap, BTreeMap};
use crate::errors::{Result, KvsError};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write, Seek, SeekFrom, BufWriter, BufReader};
use std::ffi::OsStr;
use std::ops::Range;

use serde_json::{self};
use serde::{Serialize, Deserialize};



const MAX_UNCOMPACTED_SIZE : u64 = 1024 * 1024;

/// KeyValue pairs storage engine use HashMap<String, String>
///
///
pub struct KvStore {
    path : PathBuf,
    writer : BufWriterWithPos<File>,
    readers : HashMap<u64, BufReaderWithPos<File>>,
    // index of each item
    index : BTreeMap<String, CommandPos>,
    current_gen : u64,
    // uncompacted size of the removed data
    uncompacted : u64,
}


impl KvStore {
//    /// create KvStore with empty HashMap
//    pub fn new() -> Self {
//        KvStore {
//            map : HashMap::new(),
//        }
//    }

    /// open file from specified path,
    /// build index and
    pub fn open<P>(path : P) -> Result<KvStore>
        where P : Into<PathBuf>
    {
        let path = path.into();
        let mut index = BTreeMap::new();
        let mut readers = HashMap::new();

        let gen_list = sorted_gen_list(&path)?;
        let mut uncompacted = 0 as u64;

        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        // add writer to readers
        let writer = new_log_file(&path, current_gen, &mut readers)?;

        Ok(KvStore {
            path,
            writer,
            readers,
            index,
            current_gen,
            uncompacted,
        })
    }

    fn new_log_file(&mut self, gen : u64) -> Result<BufWriterWithPos<File>> {
        new_log_file(&self.path, gen, &mut self.readers)
    }

    pub fn compact(&mut self) -> Result<()> {
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = self.new_log_file(self.current_gen)?;
        let mut compaction_writer = self.new_log_file(compaction_gen)?;

        let mut new_pos = 0;
        for cmd_pos in &mut self.index.values_mut() {
            let reader = self
                     .readers
                     .get_mut(&cmd_pos.gen)
                     .expect("Cannot find log reader");
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *cmd_pos = (compaction_gen, new_pos..new_pos + len).into();
            new_pos += len;
        }

        // remove stale readers 
        let stale_gens : Vec<_> = self
            .readers
            .keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned()
            .collect();

        for &stale_gen in &stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }

        self.uncompacted = 0;
        Ok(())
    }


    pub fn set(&mut self, key : String, value : String) -> Result<()> {
        let set_command = Command::set(key.clone(), value.clone());
        
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &set_command)?;
        self.writer.flush()?;
        let now_pos = self.writer.pos;
        
        if let Some(old_cmd) = self.index.insert(key, (self.current_gen, pos..now_pos).into()) {
            self.uncompacted += old_cmd.len;
        }

        if self.uncompacted >= MAX_UNCOMPACTED_SIZE {
            self.compact()?;
        }

        Ok(())
    }

    /// get the value from key from anything which implement the Into<String> trait
    pub fn get(&mut self, key : String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            let reader = self
                .readers
                .get_mut(&cmd_pos.gen)
                .expect("File not found");
            

            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let entry_reader = reader.take(cmd_pos.len);
            if let Command::Set{value, ..} = serde_json::from_reader(entry_reader)? {
                return Ok(Some(value));
            } else {
                return Ok(None);
            }
        }
        Ok(None)
    }


    /// remove the key-value pair from kv-storage if it exist
    pub fn remove(&mut self, key : String) -> Result<()> {
        if let None = self.index.get(&key) {
            return Err(KvsError::KeyNotFound);
        }
        let cmd = Command::remove(key.clone());

        let prev_pos = self.writer.pos; 
        serde_json::to_writer(&mut self.writer, &cmd)?;
        // BufWriter should be flushed after serialize
        self.writer.flush()?;
        let new_pos = self.writer.pos;
        let cmd_pos : CommandPos = (self.current_gen , prev_pos..new_pos).into();

        self.uncompacted += cmd_pos.len;
        if let Some(old_cmd) = self.index.insert(key, cmd_pos) {
            self.uncompacted += old_cmd.len;
        }
        Ok(())
    }
}

fn sorted_gen_list(path : &Path) -> Result<Vec<u64>>{
    let mut gen_list : Vec<u64> = fs::read_dir(path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path())})
        .filter(|path| path.is_file() && Some(OsStr::new("log")) == path.extension())
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(|s| s.parse::<u64>())
        })
        .flatten()
        .collect();

    gen_list.sort_unstable();
    Ok(gen_list)
}

fn log_path(dir : &Path, gen : u64) -> PathBuf {
    dir.join(format!("{}.log", gen))
}

fn new_log_file(
    path : &Path, 
    gen : u64, 
    readers : &mut HashMap<u64, BufReaderWithPos<File>>
) -> Result<BufWriterWithPos<File>> {
    let path = log_path(path, gen);
    let writer = BufWriterWithPos::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(&path)?
    )?;
    readers.insert(gen, BufReaderWithPos::new(File::open(path)?)?);
    Ok(writer)
}


#[derive(Serialize, Deserialize)]
pub enum Command {
    Set{
        key : String,
        value : String,
    },
    Remove {
        key : String,
    }
}

impl Command {
    fn set(key : String, value : String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key : String) -> Command {
        Command::Remove { key }
    }
}

/// Represents of the position and length of a json-serialized command in the log
pub struct CommandPos {
    // serialize number of the log
    gen : u64,
    // command len
    len : u64,

    pos : u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((gen, range) : (u64, Range<u64>)) -> Self {
        CommandPos {
            gen,
            pos : range.start,
            len : range.end - range.start,
        }
    }
}

struct BufReaderWithPos<R : Read + Seek> {
    reader : BufReader<R>,
    pos    : u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner : R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader : BufReader::new(inner),
            pos,
        })
    }
}

impl<R : Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos : SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}


impl<R : Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf : &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

struct BufWriterWithPos<W : Write + Seek> {
    writer : BufWriter<W>,
    pos     : u64,
}

impl<W : Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner : W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer : BufWriter::new(inner),
            pos,
        })
    }
}

impl<W : Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos : SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

impl<W : Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf : &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
       self.writer.flush()
    }
}


/// Load Command from specified gen log file,
/// save the each command int the index
fn load<R>(gen : u64, reader : &mut BufReaderWithPos<R>, index : &mut BTreeMap<String, CommandPos>) -> Result<u64>
    where R : Read + Seek
{
    // start pos of file
    let mut pos = 0 as u64;
    let mut stream = serde_json::Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;

    while let Some(command) = stream.next() {
        let command = command.unwrap();
        let new_pos = stream.byte_offset() as u64;

        debug_assert!(pos < new_pos, "new_pos shuld be smaller than new_pos");
        
        match command {
            Command::Set{key, ..} => {
                if let Some(old_cmd) = index.insert(key, (gen, pos..new_pos).into()) {
                    uncompacted += old_cmd.len; 
                }
            },
            Command::Remove{key} => {
                if let Some(old_cmd) = index.insert(key, (gen, pos..new_pos).into()) {
                    uncompacted += old_cmd.len;
                }
                uncompacted += new_pos - pos;
            }
        }

        pos = new_pos;
    }

    Ok(uncompacted)
}
