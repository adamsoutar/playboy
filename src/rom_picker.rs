use alloc::{vec::Vec, vec, string::String, format};
use anyhow::Error;
use crankstart::{
    file::FileSystem,
    graphics::{Graphics, LCDColor, LCDSolidColor},
    system::System,
    Playdate
};
use crankstart_sys::{FileOptions, PDButtons, LCDBitmapDrawMode};
use euclid::{point2, rect};

const X_PADDING: i32 = 20;
const Y_PADDING: i32 = 7;

pub struct RomPickerState {
  // These do *not* include the .gb postfix.
  // Eg. "Mario.gb" is listed as "Mario"
  games: Vec<String>,
  // Index of the currently selected game
  selected: usize,
  // Offsets which games we're drawing
  scroll: usize
}

impl RomPickerState {
  // If the user picks a game, this update function returns a Vec<u8> which is a
  // loaded game ROM buffer
  pub fn update (&mut self, _playdate: &mut Playdate) -> Result<Option<Vec<u8>>, Error> {
    let system = System::get();

    let (_, btns_down, _) = system.get_button_state()?;

    if 
      (btns_down & PDButtons::kButtonDown) == PDButtons::kButtonDown &&
      self.selected < self.games.len() - 1 {
      self.selected += 1;
      
      if self.selected - self.scroll >= 5 {
        // They've selected something that would push them off the screen
        self.scroll += 1;
        self.draw_whole_game_list()?;
      } else {
        // Otherwise, we can just redraw the new selection and the old selection
        // Saving screen updates, battery-life, etc.
        self.draw_game_list_item(self.selected - self.scroll)?;
        self.draw_game_list_item(self.selected - self.scroll - 1)?;
      }
    }

    if 
      (btns_down & PDButtons::kButtonUp) == PDButtons::kButtonUp &&
      self.selected > 0 {
      self.selected -= 1;

      if self.selected < self.scroll {
        self.scroll -= 1;
        self.draw_whole_game_list()?;
      } else {
        self.draw_game_list_item(self.selected - self.scroll)?;
        self.draw_game_list_item(self.selected - self.scroll + 1)?;
      }
    }

    if (btns_down & PDButtons::kButtonA) == PDButtons::kButtonA {
      // They want to select a game! 
      // Let's read it off the file system and return it
      let game_name = &self.games[self.selected];
      let path = &format!("{}.gb", game_name)[..];
      
      let file_system = FileSystem::get();

      let rom_stat = file_system.stat(path)?;
      let mut rom_buffer = vec![0; rom_stat.size as usize];

      let rom_file = file_system.open(
        path, FileOptions::kFileRead | FileOptions::kFileReadData
      )?;
      rom_file.read(&mut rom_buffer)?;

      return Ok(Some(rom_buffer))
    }

    Ok(None)
  }

  fn draw_whole_game_list (&self) -> Result<(), Error> {
    for i in 0..min(6, self.games.len()) {
      self.draw_game_list_item(i)?
    }
    Ok(())
  }

  fn draw_game_list_item (&self, index: usize) -> Result<(), Error> {
    let graphics = Graphics::get();

    let scrn_index = index as i32;
    let game_index = index + self.scroll;
    let am_selected = self.selected == game_index;

    let top = 30 + 30 * scrn_index + Y_PADDING * (scrn_index + 1);
    graphics.fill_rect(
      rect(X_PADDING, top, 400 - X_PADDING * 2, 30), 
      LCDColor::Solid(
        if am_selected { LCDSolidColor::kColorBlack } else { LCDSolidColor::kColorWhite }
      )
    )?;

    if game_index >= self.games.len() {
      return Ok(())
    }

    graphics.set_draw_mode(LCDBitmapDrawMode::kDrawModeNXOR)?;
    graphics.draw_text(&self.games[game_index][..], point2(X_PADDING + 10, top + 6))?;

    Ok(())
  }

  fn draw_initial_ui (&mut self) -> Result<(), Error> {
    let graphics = Graphics::get();

    graphics.clear(LCDColor::Solid(LCDSolidColor::kColorWhite))?;

    // Draw the initial top bar
    graphics.fill_rect(rect(0, 0, 400, 30), LCDColor::Solid(LCDSolidColor::kColorBlack))?;

    graphics.set_draw_mode(LCDBitmapDrawMode::kDrawModeInverted)?;
    graphics.draw_text("Playboy - Select a game", point2(6, 6))?;

    let file_system = FileSystem::get();

    // Find files ending with '.gb' and push them into games
    let files = file_system.listfiles(".")?;
    for filename in files {
      if filename.ends_with(".gb") {
        let game = String::from(&filename[..filename.len() - 3]);
        self.games.push(game);
      }
    }

    self.draw_whole_game_list()?;

    Ok(())
  }

  pub fn new () -> Self {
    let mut new_picker = Self {
      games: vec![],
      selected: 0,
      scroll: 0
    };

    // Calling "new" also implies you want to transition to the ROM Picker
    // right now. As such, we will draw the parts of the screen that never
    // change now to save battery life.
    new_picker.draw_initial_ui().unwrap();

    new_picker
  }
}

// I need min but not using std
fn min (x: usize, y: usize) -> usize {
  if x > y { y } else { x }
}
