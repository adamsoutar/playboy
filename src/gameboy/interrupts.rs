pub const INTERRUPT_VECTORS: [u16; 5] = [0x40, 0x48, 0x50, 0x58, 0x60];

#[derive(Clone)]
pub struct InterruptFields {
    pub v_blank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool
}

pub enum InterruptReason {
    VBlank,
    LCDStat,
    Timer,
    Serial,
    Joypad
}
fn get_interrupt_reason_bitmask (reason: InterruptReason) -> u8 {
    match reason {
        InterruptReason::VBlank => 0b00000001,
        InterruptReason::LCDStat => 0b00000010,
        InterruptReason::Timer => 0b00000100,
        InterruptReason::Serial => 0b00001000,
        InterruptReason::Joypad => 0b00010000,
    }
}

impl InterruptFields {
    // TODO: Check if these actually do all start false
    pub fn new () -> InterruptFields {
        InterruptFields {
            v_blank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false
        }
    }
}
impl From<u8> for InterruptFields {
    fn from(n: u8) -> InterruptFields {
        InterruptFields {
            v_blank: (n & 1) == 1,
            lcd_stat: ((n >> 1) & 1) == 1,
            timer: ((n >> 2) & 1) == 1,
            serial: ((n >> 3) & 1) == 1,
            joypad: ((n >> 4) & 1) == 1
        }
    }
}
impl From<InterruptFields> for u8 {
    fn from(f: InterruptFields) -> u8 {
        let b1 = f.v_blank as u8;
        let b2 = (f.lcd_stat as u8) << 1;
        let b3 = (f.timer as u8) << 2;
        let b4 = (f.serial as u8) << 3;
        let b5 = (f.joypad as u8) << 4;
        b1 | b2 | b3 | b4 | b5
    }
}

pub struct Interrupts {
    pub enable: InterruptFields,
    pub flag: InterruptFields,

    // "Interrupts master enabled" flag
    pub ime: bool
}

impl Interrupts {
    pub fn raise_interrupt (&mut self, reason: InterruptReason) {
        let mut data = self.flag_read();
        data |= get_interrupt_reason_bitmask(reason);
        self.flag_write(data);
    }

    // Called when GB writes to FFFF
    pub fn enable_write (&mut self, value: u8) {
        // println!("{:08b} written to IE", value);
        self.enable = InterruptFields::from(value)
    }

    // Called when GB writes to FF0F
    pub fn flag_write (&mut self, value: u8) {
        self.flag = InterruptFields::from(value)
    }

    // Called when GB reads from FFFF
    pub fn enable_read (&self) -> u8 {
        u8::from(self.enable.clone())
    }

    // Called when GB reads from FF0F
    pub fn flag_read (&self) -> u8 {
        u8::from(self.flag.clone())
    }

    pub fn new () -> Interrupts {
        Interrupts {
            enable: InterruptFields::new(),
            flag: InterruptFields::new(),

            ime: false
        }
    }
}
