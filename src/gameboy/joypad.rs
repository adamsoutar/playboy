enum JoypadReadoutMode {
    Buttons,
    Directions,
    Neither
  }
  
  // TODO: Raise the Joypad interrupt
  pub struct Joypad {
    readout_mode: JoypadReadoutMode,
  
    // The GUI writes these values directly via the keyboard
    // Every frame.
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub a_pressed: bool,
    pub b_pressed: bool,
    pub start_pressed: bool,
    pub select_pressed: bool
  }
  
  impl Joypad {
    pub fn write (&mut self, n: u8) {
        let masked = n & 0b0011_0000;
  
        self.readout_mode = match masked {
            0b0001_0000 => JoypadReadoutMode::Buttons,
            0b0010_0000 => JoypadReadoutMode::Directions,
            _ => JoypadReadoutMode::Neither
        }
    }
  
    fn direction_bits (&self) -> u8 {
        (!self.right_pressed as u8) |
        ((!self.left_pressed as u8) << 1) |
        ((!self.up_pressed as u8) << 2) |
        ((!self.down_pressed as u8) << 3)
    }
  
    fn button_bits (&self) -> u8 {
        (!self.a_pressed as u8) |
        ((!self.b_pressed as u8) << 1) |
        ((!self.select_pressed as u8) << 2) |
        ((!self.start_pressed as u8) << 3)
    }
  
    fn selection_bits (&self) -> u8 {
        match self.readout_mode {
            JoypadReadoutMode::Buttons => 0b0010_0000,
            JoypadReadoutMode::Directions => 0b0001_0000,
            JoypadReadoutMode::Neither => 0
        }
    }
  
    pub fn read (&self) -> u8 {
        let n = match self.readout_mode {
            JoypadReadoutMode::Buttons => self.button_bits(),
            JoypadReadoutMode::Directions => self.direction_bits(),
            JoypadReadoutMode::Neither => 0xF
        };
  
        n | self.selection_bits()
    }
  
    pub fn new () -> Joypad {
        Joypad {
            readout_mode: JoypadReadoutMode::Buttons,
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            a_pressed: false,
            b_pressed: false,
            start_pressed: false,
            select_pressed: false
        }
    }
  }
  