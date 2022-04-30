#![no_std]

extern crate alloc;

use core::ffi::c_void;

use alloc::{boxed::Box, vec, format};
use anyhow::Error;
use crankstart::{
    crankstart_game, file::FileSystem,
    graphics::{Graphics, LCDColor, LCDSolidColor},
    system::System,
    Game, Playdate, log_to_console
};
use crankstart_sys::{FileOptions, PDButtons, LCD_ROWS};
use euclid::num::Floor;

use gbrs_core::{callbacks::*, constants::*, cpu::Cpu, lcd::GreyShade};

mod rom_picker;
use rom_picker::RomPickerState;

// On hardware, we'll target 15 FPS, which is more achievable, and still
// playable.
// On the simulator, we should have more than enough power to push 30 FPS.
// Hopefully this can be universally set to 30 one day when everything is fast
// enough.
#[cfg(any(target_os = "windows", target_os = "macos"))]
const FRAME_RATE: usize = 30;
#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
const FRAME_RATE: usize = 15;

// This is how much we'll scale the Gameboy screen to fit it on the Playdate
const SCALE_FACTOR: f32 = 1.6666666667;

// This is nasty and not very Rust-like, but it's about the best I can manage
// given that I need to mutate state via an extern C fn.
// TODO: Do this a different way
static mut WANT_TO_QUIT_GAME: bool = false;

struct State {
    processor: Option<Cpu>,
    // This is used to determine when the crank has changed direction
    // (we use that for Start/Select)
    last_crank_change: f32,
    rom_picker: Option<RomPickerState>
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        crankstart::display::Display::get().set_refresh_rate(FRAME_RATE as f32)?;

        unsafe {
            set_callbacks(Callbacks {
                log: |log_str| log_to_console!("{}", log_str),
                save: |game_name, _rom_path, save_data| {
                    let file_system = FileSystem::get();
                    let save_path = &format!("{}.sav", game_name)[..];
                    let save_file = file_system
                        .open(
                            save_path,
                            FileOptions::kFileWrite
                        ).unwrap();
                    save_file.write(&save_data[..]).unwrap();
                },
                load: |game_name, _rom_path, expected_size| {
                    let file_system = FileSystem::get();
                    let save_path = &format!("{}.sav", game_name)[..];

                    let stat_result = file_system.stat(save_path);

                    if let Ok(stat) = stat_result {
                        // There is a save file and we can read it!
                        // NOTE: stat.size might not be the expected_size, but
                        //   that error-case is already handled in gbrs' ram.rs
                        let mut buffer = vec![0; stat.size as usize];
                        let save_file = file_system
                            .open(
                                save_path,
                                FileOptions::kFileRead | FileOptions::kFileReadData
                            ).unwrap();
                        save_file.read(&mut buffer).unwrap();
                        log_to_console!("Loaded {}", save_path);
                        buffer
                    } else {
                        // Error at that path, there probably just isn't a save
                        //   file yet. Return all 0s
                        // TODO: Should this be all 0 or all 0xFF?
                        log_to_console!("{} not found", save_path);
                        vec![0; expected_size]
                    }
                }
            })
        }

        // Let's write a handy little helper file to point new folk in the
        // right direction.
        let file_system = FileSystem::get();
        let help_file = file_system
            .open(
                "Game ROMs go here",
                FileOptions::kFileWrite
            ).unwrap();
        help_file.write(&[]).unwrap();

        // Provide a menu item for going back to the rom picker
        // Allowing you to quit a game without quitting Playboy
        let system = System::get();

        unsafe extern "C" fn quit_game_callback (_: *mut c_void) {
            WANT_TO_QUIT_GAME = true;
        }
        system.add_menu_item("quit game", Some(quit_game_callback))?;

        Ok(Box::new(Self {
            processor: None,
            last_crank_change: 0.,
            rom_picker: Some(RomPickerState::new())
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

impl Game for State {
    fn update(&mut self, playdate: &mut Playdate) -> Result<(), Error> {
        let system = System::get();
        let graphics = Graphics::get();

        unsafe {
            if WANT_TO_QUIT_GAME {
                WANT_TO_QUIT_GAME = false;
                self.rom_picker = Some(RomPickerState::new());
                self.processor = None;
            }
        }

        if let Some(rom_picker) = &mut self.rom_picker {
            let maybe_picked_game = rom_picker.update(playdate)?;

            if let Some(picked_game) = maybe_picked_game {
                let mut cpu = Cpu::from_rom_bytes(picked_game);
                cpu.frame_rate = FRAME_RATE;
                self.processor = Some(cpu);
                self.rom_picker = None;
                graphics.clear(LCDColor::Solid(LCDSolidColor::kColorBlack))?;
            // Else they're still picking
            } else { return Ok(()) }
        }

        let gameboy = self.processor.as_mut().unwrap();

        let crank_change = system.get_crank_change()?;
        let processed_crank =
            process_crank_change(crank_change, self.last_crank_change);
        self.last_crank_change = crank_change;

        let (btns_held, _, _) = system.get_button_state()?;

        // TODO: Raise the joypad interrupt
        gameboy.mem.joypad.a_pressed =
            (btns_held & PDButtons::kButtonA) == PDButtons::kButtonA;
        gameboy.mem.joypad.b_pressed =
            (btns_held & PDButtons::kButtonB) == PDButtons::kButtonB;
        gameboy.mem.joypad.up_pressed =
            (btns_held & PDButtons::kButtonUp) == PDButtons::kButtonUp;
        gameboy.mem.joypad.down_pressed =
            (btns_held & PDButtons::kButtonDown) == PDButtons::kButtonDown;
        gameboy.mem.joypad.left_pressed =
            (btns_held & PDButtons::kButtonLeft) == PDButtons::kButtonLeft;
        gameboy.mem.joypad.right_pressed =
            (btns_held & PDButtons::kButtonRight) == PDButtons::kButtonRight;
        gameboy.mem.joypad.start_pressed = processed_crank > 0.;
        gameboy.mem.joypad.select_pressed = processed_crank < 0.;

        // Actually *run* the Gameboy game.
        gameboy.step_one_frame();

        // Draw screen
        let playdate_x_pixels =
            (SCREEN_WIDTH as f32 * SCALE_FACTOR).floor() as usize;
        let playdate_y_pixels = LCD_ROWS as usize;

        // I've got a speculation that writing in X rows is better because
        // that's how the framebuffer is written out in memory, but I'm not
        // sure.
        let framebuffer = graphics.get_frame()?;

        for y in 0..playdate_y_pixels {
            let mut screen_byte: u8 = 0x00;
            for x in 0..playdate_x_pixels {
                let gameboy_x = (x as f32 / SCALE_FACTOR).floor() as usize;
                let gameboy_y = (y as f32 / SCALE_FACTOR).floor() as usize;
                let gameboy_lcd_index = gameboy_y * SCREEN_WIDTH + gameboy_x;
                let shade_at =
                    &gameboy.gpu.finished_frame[gameboy_lcd_index];

                let bit_index = 7 - (x % 8);

                match shade_at {
                    GreyShade::Black => {
                        // The screen_byte is already black by default
                    },
                    GreyShade::DarkGrey => {
                        // Same as below but draws every 3 pixels rather than 2
                        let should_be_white = (x + y % 2) % 3 == 0;
                        if should_be_white {
                            screen_byte |= 1 << bit_index;
                        }
                    },
                    GreyShade::LightGrey => {
                        // This is a frame-stable cross-hatching calculation
                        // On even Y rows, we draw pixels on every even X coord,
                        // On odd Y rows, we draw pixels on every odd X coord
                        let should_be_white = (x + y % 2) % 2 == 0;
                        if should_be_white {
                            screen_byte |= 1 << bit_index;
                        }
                    },
                    GreyShade::White => {
                        screen_byte |= 1 << bit_index;
                    }
                }

                if (x + 1) % 8 == 0 {
                    // We've drawn a row of 8 pixels, let's commit it to the
                    // frame buffer.
                    // x + 64 is to horizontally center(-ish) the screen.
                    // 67 is the actual constant, but keeping it divisible by
                    // 8 lets us do faster framebuffer maths.
                    let byte_index = y * 52 + (x + 64) / 8;
                    framebuffer[byte_index] = screen_byte;
                    screen_byte = 0x00;
                }
            }
        }

        // NOTE: This redraws the entire scren. Here we lose our little
        //   optimisation we had before where we wouldn't redraw the borders
        //   around the gameboy screen.
        // TODO: Would it be quicker to check if screen_byte is going to be the
        //   same as it was last frame, and then try and skip some updated
        //   rows?
        graphics.mark_updated_rows(0..=(LCD_ROWS - 1) as i32)?;

        Ok(())
    }
}

crankstart_game!(State);
