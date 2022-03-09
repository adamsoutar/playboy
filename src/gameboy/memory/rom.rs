// use std::io::Read;
// use std::fs::File;
use alloc::vec;
use alloc::vec::Vec;

pub struct Rom {
    pub bytes: Vec<u8>
}

impl Rom {
    pub fn read (&self, address: u16) -> u8 {
        self.bytes[address as usize]
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

    pub fn from_hardcoded_tetris () -> Rom {
        let tetris_bytes = include_bytes!("./tetris.gb");
        Rom {
            bytes: tetris_bytes.to_vec()
        }
    }
}
