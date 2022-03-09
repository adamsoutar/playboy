// Gameboy Link Cable
// Mostly a stub, although some parts have to be emulated *somewhat* accurately
// to emulate fussy games like Alleyway
use crate::gameboy::constants::*;
use crate::gameboy::interrupts::{Interrupts, InterruptReason};

// Unusual serial code inspired by
// https://github.com/rvaccarim/FrozenBoy/blob/master/FrozenBoyCore/Serial/SerialLink.cs

pub struct SerialCable {
  transfer_data_byte: u8,
  transfer_control_byte: u8,

  counter: usize,
  transfer_in_progress: bool
}

impl SerialCable {
  pub fn read (&self, address: u16) -> u8 {
    match address {
      // When there's no gameboy on the other end, this apparently
      // just always reads 0xFF
      LINK_CABLE_SB => self.transfer_data_byte,
      LINK_CABLE_SC => self.transfer_control_byte | 0b1111_1110,
      _ => unreachable!()
    }
  }

  pub fn write (&mut self, address: u16, value: u8) {
    match address {
      LINK_CABLE_SB => self.transfer_data_byte = value,
      LINK_CABLE_SC => {
        self.transfer_control_byte = value;

        if value == 0x81 {
          self.transfer_in_progress = true;
          self.counter = 0;
        }
      },
      _ => unreachable!()
    }
  }

  pub fn step (&mut self, ints: &mut Interrupts) {
    if !self.transfer_in_progress { return; }

    self.counter += 1;

    if self.counter >= 512 {
      self.transfer_in_progress = false;
      self.transfer_data_byte = 0xFF;
      ints.raise_interrupt(InterruptReason::Serial);
    }
  }

  pub fn new () -> SerialCable {
    SerialCable {
      transfer_data_byte: 0,
      transfer_control_byte: 0,

      counter: 0,
      transfer_in_progress: false
    }
  }
}