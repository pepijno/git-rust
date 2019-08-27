extern crate byteorder;

use std::io::Read;
use byteorder::{ReadBytesExt, BigEndian};

fn sha1_to_hex(sha1: [u8; 20]) -> String {
    let strings: Vec<String> = sha1.iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    strings.join("")
}

#[derive(Debug)]
struct Entry {
    sha1: [u8; 20],
    crc: u32,
    off: u32,
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            sha1: [0u8; 20],
            crc: 0u32,
            off: 0u32,
        }
    }
}

fn main() {
    let mut f = std::io::stdin();
    let header = match f.read_u32::<BigEndian>() {
        Ok(header) => header,
        Err(_e) => panic!("unable to read header")
    };
    if header != 0xFF744F63 {
        panic!("unknown index version");
    }
    let version = match f.read_u32::<BigEndian>() {
        Ok(version) => version,
        Err(_e) => panic!("unable to read version")
    };
    if version != 2 {
        panic!("unknown index version");
    }
    let mut nr = 0;
    for _ in 0..256 {
        let res = match f.read_u32::<BigEndian>() {
            Ok(res) => res,
            Err(_e) => panic!("corrupt index file")
        };
        if res < nr {
            panic!("corrupt index file");
        };
        nr = res;
    }
    let mut off64_nr = 0u32;
    let mut entries = Vec::new();
    for _ in 0..nr {
        entries.push(Entry::default());
    }
    let mut i = 0;
    entries = entries.iter().map(|entry| {
        let mut sha1 = [0u8; 20];
        let read = f.read(&mut sha1);
        if read.is_err() {
            panic!("unable to read sha1 {}/{}", i, nr);
        }
        i = i + 1;
        Entry { sha1, ..*entry }
    }).collect();
    i = 0;
    entries = entries.iter().map(|entry| {
        let crc = match f.read_u32::<BigEndian>() {
            Ok(crc) => crc,
            Err(_e) => panic!("unable to read crc {}/{}", i, nr)
        };
        Entry { crc, ..*entry }
    }).collect();
    i = 0;
    entries = entries.iter().map(|entry| {
        let off = match f.read_u32::<BigEndian>() {
            Ok(off) => off,
            Err(_e) => panic!("unable to read 32b offset {}/{}", i, nr)
        };
        Entry { off, ..*entry }
    }).collect();
    for entry in entries {
        let off = entry.off;
        let offset = if (off & 0x80000000) == 0 {
            off as u64
        } else {
            if (off & 0x7fffffff) != off64_nr {
                panic!("inconsistent 64b offset index");
            }
            let res = match f.read_u64::<BigEndian>() {
                Ok(res) => res,
                Err(_e) => panic!("unable to read 64b offset {}", off64_nr)
            };
            off64_nr = off64_nr + 1;
            res
        };
        println!("{} {} ({:x?})", offset, sha1_to_hex(entry.sha1), entry.crc);
    }
}
