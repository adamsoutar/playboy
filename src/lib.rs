#![no_std]

extern crate alloc;

use {
    alloc::{
        format,
        boxed::Box
    },
    anyhow::Error,
    crankstart::{
        crankstart_game,
        geometry::{ScreenPoint, ScreenVector},
        graphics::{Graphics, LCDColor, LCDSolidColor},
        system::System,
        Game, Playdate,
    },
    crankstart_sys::{LCD_COLUMNS, LCD_ROWS},
    euclid::{point2, vec2},
};

mod gameboy;
use gameboy::cpu::Cpu;
use gameboy::constants::CYCLES_PER_FRAME;

struct State {
    processor: Cpu
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(30.0)?;
        
        let mut cpu = Cpu::from_hardcoded_tetris();

        Ok(Box::new(Self {
            processor: cpu
        }))
    }
}

impl Game for State {
    fn update(&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
        let graphics = Graphics::get();
        graphics.clear(LCDColor::Solid(LCDSolidColor::kColorWhite))?;

        System::get().draw_fps(0, 0)?;
        graphics.draw_text(&format!("Running: {}", self.processor.cart_info.title)[..], point2(20,20))?;
        graphics.draw_text("Debug registers:", point2(20,60))?;
        graphics.draw_text(&format!("A: {} B: {} C: {}", self.processor.regs.a, self.processor.regs.b, self.processor.regs.c)[..], point2(20,80))?;

        for _ in 0..CYCLES_PER_FRAME {
            self.processor.step();
        }

        Ok(())
    }
}

crankstart_game!(State);
