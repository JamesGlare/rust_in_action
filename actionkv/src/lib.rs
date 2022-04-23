#[macro_use]
extern crate serde_derive;

extern crate byteorder;
extern crate crc;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, SeekFrom};
use std::path::Path;

type ByteString = Vec<u8>;
type ByteStr = [u8];

#[derive(Debug, Serialize, Deserialize)] // #[derive(Debug)]
pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}

#[derive(Debug)] // #[derive(Debug)]
pub struct ActionKV {
    f: File,
    pub index: HashMap<ByteString, u64>,
}

impl ActionKV {
    pub fn open(path: &Path) -> io::Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path)?;
        let index = HashMap::new();
        return Ok(ActionKV { f, index });
    }
}

fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
    let saved_checksum = f.read_u32::<LittleEndian>()?;
    let key_len = f.read_u32::<LittleEndian>()?;
    let val_len = f.read_u32::<LittleEndian>()?;
    let data_len = key_len + val_len;
    let mut data = ByteString::with_capacity(data_len as usize);
    {
        f.by_ref().take(data_len as u64).read_to_end(&mut data)?;
    }
    debug_assert_eq!(data.len(), data_len as usize);

    let checksum = crc32::checksum_ieee(&data);
    if checksum != saved_checksum {
        panic!(
            "data corruption encountered ({:08x} != {:08x}",
            checksum, saved_checksum
        );
    }

    let value = data.split_off(key_len as usize);
    let key = data;
    return Ok(KeyValuePair { key, value });
}
