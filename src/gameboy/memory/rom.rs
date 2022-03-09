// use std::io::Read;
// use std::fs::File;
use alloc::vec;
use alloc::vec::Vec;

pub struct Rom {
    pub bytes: Vec<u8>
}

impl Rom {
    pub fn read (&self, address: u16) -> u8 {
        // TODO: PLAYDATE: This is a bodge. gbrs doesn't do this
        if address as usize >= self.bytes.len() { 0 }
        else { self.bytes[address as usize] }
    }

    pub fn from_file (path: &str) -> Rom {
        let mut buffer = vec![];
        // let mut file = File::open(path).expect("Invalid ROM path");
        // file.read_to_end(&mut buffer).expect("Unable to read ROM file");
        // TODO: PLAYDATE: Read files

        Rom {
            bytes: buffer
        }
    }

    pub fn from_playdate_bodge_zeroes () -> Rom {
        Rom {
            bytes: vec![0; 100]
        }
    }
}
