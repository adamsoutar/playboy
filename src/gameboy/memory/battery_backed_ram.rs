// RAM with a save file
// use std::fs;
// use std::path::{Path, PathBuf};
// use std::time::SystemTime;
use crate::gameboy::memory::ram::Ram;
use alloc::string::String;

// The amount of seconds we wait before saving our save file
// (otherwise eg. Link's Awakening would write 2,700 save files 
//  on its first frame)
const DEBOUNCE_SECONDS: u64 = 1;

fn get_save_file_path (rom_path: &str) -> String {
  // let mut sav_path = PathBuf::from(rom_path);
  // sav_path.set_extension("sav");

  // sav_path
  //   .to_string_lossy()
  //   .to_string()
  // TODO: PLAYDATE: Actually save "battery backed" RAM
  String::new()
}

pub struct BatteryBackedRam {
  ram: Ram,
  pub size: usize,

  save_file_path: String,
  battery_enabled: bool,
  // last_saved_at: SystemTime, TODO: PLAYDATE
  changed_since_last_save: bool,
}

impl BatteryBackedRam {
  pub fn read (&self, address: u16) -> u8 {
    self.ram.read(address)
  }

  pub fn write (&mut self, address: u16, value: u8) {
    self.ram.write(address, value);
    self.changed_since_last_save = true;
  }

  pub fn step (&mut self) {
    if !self.changed_since_last_save || !self.battery_enabled { return }

    // let seconds_since_last_save = self.last_saved_at
    //   .elapsed()
    //   .unwrap()
    //   .as_secs();

    // if seconds_since_last_save >= DEBOUNCE_SECONDS {
    //   self.save_ram_contents()
    // }
  }

  // fn save_ram_contents (&mut self) {
  //   self.last_saved_at = SystemTime::now();
  //   self.changed_since_last_save = false;
    
  //   fs::write(&self.save_file_path, &self.ram.bytes)
  //     .expect("Failed to write save file")
  // }

  pub fn new (size: usize, battery_enabled: bool, rom_path: &str) -> BatteryBackedRam {
    let save_file_path = get_save_file_path(rom_path);

    let ram: Ram;

    // TODO: PLAYDATE: Load save files
    // if Path::new(&save_file_path).exists() {
    //   // There is an existing save file for this game,
    //   // load it in
    //   ram = Ram::from_file(&save_file_path, size)
    // } else { 
      ram = Ram::new(size);
    // }

    BatteryBackedRam {
      ram,
      size,

      save_file_path,
      battery_enabled,
      // last_saved_at: SystemTime::now(),
      changed_since_last_save: false
    }
  } 
}
