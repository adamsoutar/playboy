use alloc::{boxed::Box, vec, format};
use anyhow::Error;
use crankstart::{
    crankstart_game, file::FileSystem,
    graphics::{Graphics, LCDColor, LCDSolidColor, Font},
    system::System,
    Game, Playdate, log_to_console,
};
use crankstart_sys::{FileOptions, PDButtons, LCD_ROWS, LCDBitmapDrawMode};
use euclid::{num::Floor, point2, rect};


pub struct RomPickerState {

}

impl RomPickerState {
  pub fn update (&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
    Ok(())
  }

  fn draw_initial_ui (&mut self) -> Result<(), Error> {
    let graphics = Graphics::get();

    graphics.clear(LCDColor::Solid(LCDSolidColor::kColorWhite))?;

    graphics.fill_rect(rect(0, 0, 400, 30), LCDColor::Solid(LCDSolidColor::kColorBlack))?;

    graphics.set_draw_mode(LCDBitmapDrawMode::kDrawModeInverted)?;
    graphics.draw_text("Playboy - Select a game", point2(6, 6))?;

    Ok(())
  }

  pub fn new () -> Self {
    let mut new_picker = Self {};

    // Calling "new" also implies you want to transition to the ROM Picker
    // right now. As such, we will draw the parts of the screen that never
    // change now to save battery life.
    new_picker.draw_initial_ui().unwrap();

    new_picker
  }
}
