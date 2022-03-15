#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use anyhow::Error;
use crankstart::{
    crankstart_game,
    geometry::ScreenRect,
    graphics::{Graphics, LCDColor, LCDSolidColor},
    system::System,
    Game, Playdate
};
use crankstart_sys::{PDButtons, LCD_ROWS};
use euclid::{num::Floor, point2, size2};

use gbrs_core::{callbacks::*, constants::*, cpu::Cpu, lcd::GreyShade};

// The Playdate LCD actually updates at half the rate of the Gameboy
const FRAME_RATE: usize = 30;
// This is how much we'll scale the Gameboy screen to fit it on the Playdate
const SCALE_FACTOR: f32 = 1.6666666667;
// Start the image at this x coordinate (centers the scaled image)
const START_X: i32 = 67;

struct State {
    processor: Cpu,
    // This is used to determine when the crank has changed direction
    // (we use that for Start/Select)
    last_crank_change: f32
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(30.0)?;
        Graphics::get().clear(LCDColor::Solid(LCDSolidColor::kColorBlack))?;

        unsafe {
            set_callbacks(Callbacks {
                log: |log_str| System::log_to_console(log_str)
            })
        }

        let mut cpu = Cpu::from_rom_bytes(include_bytes!("../rom.gb").to_vec());
        cpu.frame_rate = FRAME_RATE;

        Ok(Box::new(Self {
            processor: cpu,
            last_crank_change: 0.
        }))
    }
}

// This is kind of like a differential.
// We're looking for a "change in change" in crank angle
fn process_crank_change(new_crank: f32, old_crank: f32) -> f32 {
    // Is this safe with floats? (no epsilon etc.)
    if old_crank > 0. && new_crank > 0. {
        0.
    } else if old_crank < 0. && new_crank < 0. {
        0.
    } else if new_crank == 0. {
        0.
    } else {
        new_crank
    }
}

fn draw_pixel_at(
    graphics: &Graphics,
    x: usize,
    y: usize,
    white: bool
) -> Result<(), Error> {
    graphics.fill_rect(
        ScreenRect::new(point2(START_X + x as i32, y as i32), size2(1, 1)),
        LCDColor::Solid(if white {
            LCDSolidColor::kColorWhite
        } else {
            LCDSolidColor::kColorBlack
        })
    )
}

impl Game for State {
    fn update(&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
        let system = System::get();
        let graphics = Graphics::get();

        let crank_change = system.get_crank_change()?;
        let processed_crank =
            process_crank_change(crank_change, self.last_crank_change);
        self.last_crank_change = crank_change;

        let (btns_held, _, _) = system.get_button_state()?;

        // TODO: Raise the joypad interrupt
        self.processor.mem.joypad.a_pressed =
            (btns_held & PDButtons::kButtonA) == PDButtons::kButtonA;
        self.processor.mem.joypad.b_pressed =
            (btns_held & PDButtons::kButtonB) == PDButtons::kButtonB;
        self.processor.mem.joypad.up_pressed =
            (btns_held & PDButtons::kButtonUp) == PDButtons::kButtonUp;
        self.processor.mem.joypad.down_pressed =
            (btns_held & PDButtons::kButtonDown) == PDButtons::kButtonDown;
        self.processor.mem.joypad.left_pressed =
            (btns_held & PDButtons::kButtonLeft) == PDButtons::kButtonLeft;
        self.processor.mem.joypad.right_pressed =
            (btns_held & PDButtons::kButtonRight) == PDButtons::kButtonRight;
        self.processor.mem.joypad.start_pressed = processed_crank > 0.;
        self.processor.mem.joypad.select_pressed = processed_crank < 0.;

        // Actually *run* the Gameboy game.
        self.processor.step_one_frame();

        // Draw screen
        // TODO: While drawing 1x1 rects might work in the simulator, it may
        //   not be performant on-device.
        let playdate_x_pixels =
            (SCREEN_WIDTH as f32 * SCALE_FACTOR).floor() as usize;
        let playdate_y_pixels = LCD_ROWS as usize;

        for x in 0..playdate_x_pixels {
            for y in 0..playdate_y_pixels {
                let gameboy_x = (x as f32 / SCALE_FACTOR).floor() as usize;
                let gameboy_y = (y as f32 / SCALE_FACTOR).floor() as usize;
                let gameboy_lcd_index = gameboy_y * SCREEN_WIDTH + gameboy_x;
                let shade_at =
                    &self.processor.gpu.finished_frame[gameboy_lcd_index];

                match shade_at {
                    GreyShade::Black => {
                        draw_pixel_at(&graphics, x, y, false)?;
                    },
                    GreyShade::DarkGrey => {
                        // Same as below but draws every 3 pixels rather than 2
                        let should_be_white = (x + y % 2) % 3 == 0;
                        draw_pixel_at(&graphics, x, y, should_be_white)?;
                    },
                    GreyShade::LightGrey => {
                        // This is a frame-stable cross-hatching calculation
                        // On even Y rows, we draw pixels on every even X coord,
                        // On odd Y rows, we draw pixels on every odd X coord
                        let should_be_white = (x + y % 2) % 2 == 0;
                        draw_pixel_at(&graphics, x, y, should_be_white)?;
                    },
                    GreyShade::White => {
                        draw_pixel_at(&graphics, x, y, true)?;
                    }
                }
            }
        }

        Ok(())
    }
}

crankstart_game!(State);
