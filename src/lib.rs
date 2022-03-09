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
        geometry::{ScreenPoint, ScreenVector, ScreenRect},
        graphics::{Graphics, LCDColor, LCDSolidColor},
        system::System,
        Game, Playdate,
    },
    crankstart_sys::{LCD_COLUMNS, LCD_ROWS},
    euclid::{point2, vec2, size2},
};

mod gameboy;
use gameboy::cpu::Cpu;
use gameboy::lcd::GreyShade;
use gameboy::constants::{
    CYCLES_PER_FRAME, SCREEN_BUFFER_SIZE,
    SCREEN_WIDTH, SCREEN_HEIGHT
};

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

        // System::get().draw_fps(0, 0)?;
        // graphics.draw_text(&format!("Running: {}", self.processor.cart_info.title)[..], point2(20,20))?;
        // graphics.draw_text("Debug registers:", point2(20,60))?;
        // graphics.draw_text(&format!("A: {} B: {} C: {}", self.processor.regs.a, self.processor.regs.b, self.processor.regs.c)[..], point2(20,80))?;

        let mut cycles = 0;
        while cycles < CYCLES_PER_FRAME {
            cycles += self.processor.step();
        }

        // Draw screen
        // NOTE: This is not performant. We're just experimenting here.
        for i in 0..SCREEN_BUFFER_SIZE {
            let x = i % SCREEN_WIDTH;
            let y = i / SCREEN_WIDTH;
            let shade_at = &self.processor.gpu.finished_frame[i];

            match shade_at {
                // Not dark enough to draw in 1-bit
                GreyShade::White => {},
                GreyShade::LightGrey => {},
                _ => {
                    // Dark enough!
                    graphics.draw_rect(
                        ScreenRect::new(point2(x as i32, y as i32), size2(1, 1)),
                        LCDColor::Solid(LCDSolidColor::kColorBlack)
                    );
                }
            }
        }

        Ok(())
    }
}

crankstart_game!(State);
