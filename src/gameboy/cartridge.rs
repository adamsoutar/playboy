// Parses the cartridge header
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

#[derive(Clone)]
pub struct Cartridge {
    pub title: String,
    pub rom_path: String,
    pub cart_type: u8,

    pub rom_size: usize,
    pub ram_size: usize
}

impl Cartridge {
    pub fn parse(buffer: &Vec<u8>, rom_path: String) -> Cartridge {
        let title = get_title(buffer);

        let cart_type = buffer[0x0147];

        let rom_size_id = buffer[0x0148];
        let ram_size_id = buffer[0x0149];

        let rom_size = 32768 << (rom_size_id as usize);
        let ram_size = match ram_size_id {
            0 => 0,
            1 => {
                // println!("[WARN] Unofficial 2KB RAM size not used by any officially published game.");
                2_048
            },
            2 => 8_192,
            3 => 32_768,
            4 => 131_072,
            5 => 65_536,
            _ => panic!("Unknown RAM size id for cartridge {:#04x}", ram_size_id)
        };

        Cartridge {
            title, rom_path, cart_type, rom_size, ram_size
        }
    }

    pub fn playdate_bodge_noop () -> Cartridge {
        Cartridge {
            title: String::from("PLAYBOYTEST"),
            rom_path: String::new(),
            cart_type: 0,
            rom_size: 100,
            ram_size: 0
        }
    }
}

fn get_title (buffer: &Vec<u8>) -> String {
    let mut out_buff = vec![];
    for i in 0x0134..=0x0143 {
        // A null byte terminates the title string
        if buffer[i] == 0 { break; }
        out_buff.push(buffer[i]);
    }
    String::from_utf8(out_buff)
        .expect("ROM title isn't valid UTF-8")
}
