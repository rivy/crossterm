//! This module contains all the logic for switching between alternate screen and main screen.
//!
//!
//! *Nix style applications often utilize an alternate screen buffer, so that they can modify the entire contents of the buffer, without affecting the application that started them.
//! The alternate buffer is exactly the dimensions of the window, without any scrollback region.
//! For an example of this behavior, consider when vim is launched from bash.
//! Vim uses the entirety of the screen to edit the file, then returning to bash leaves the original buffer unchanged.
//!
//!
//! When using alternate screen there is one thing to keep in mind.
//! To get the functionalities of `cursor, color, terminal` also working on alternate screen.
//! You need to pass it the same `Context` as you have passed to the previous three functions,
//! If you don't use the same `Context` the `cursor(), color(), terminal()` these modules will be using main screen to write to.
//! So you will see nothing on alternate screen.
//!
//!
//! When you want to switch to alternate screen there are a couple of things to keep in mind for it to work correctly.
//! First off some code of how to switch to Alternate screen, for more info check the example folder at github
//!
//!
//! Create alternate screen from `Context`
//!
//!     // create context.
//!     let context = crossterm::Context::new();
//!     // create instance of Alternatescreen by the given context, this wil also switch to it.
//!     let mut screen = crossterm::AlternateScreen::from(context.clone());
//!     // write to the alternate screen.
//!     write!(screen,  "test");
//!
//! Create alternate screen from `Crossterm`:
//!
//!     // create crossterm.
//!     let crossterm = ::crossterm::Crossterm::new();
//!     // create instance of Alternatescreen by the given refrence to crossterm, this wil also switch to it.
//!     let mut screen = crossterm::AlternateScreen::from(&crossterm);
//!     // write to the alternate screen.
//!     write!(screen,  "test");
//!
//! When using alternate screen there is one thing to keep in mind.
//! To get the functionalities of `cursor, color, terminal` also working on alternate screen.
//! You need to pass it the same `Context` as you have passed to the previous three functions,
//! If you don't use the same `Context` the `cursor(), color(), terminal()` these modules will be using main screen to write to.
//! So you will see nothing on alternate screen.
//!
//!
//! Please check the documentation of `Context` for more info.
//! But basically this Context is a wrapper for a type that provides access to the current screen whether it would be the main screen or alternate screen.
//!
//!
//! An example of what I mean by that:
//!
//!     // create context.
//!     let context = crossterm::Context::new();
//!
//!     let mut cursor = ::crossterm::cursor::cursor(&context);
//!     cursor.goto(10,10);
//!
//!     // create instance of Alternatescreen by the given refrence to crossterm, this wil also switch to it.
//!     let mut screen = crossterm::AlternateScreen::from(&context);
//!
//!     // now this cursor will be moving on the alternate screen sice it is using the same context as we have passed to the alternatescreen.
//!     cursor.goto(5,4)
//!
//! To make things easier you can better use `Crossterm` type for the interactions with alternate screen.
//! This type will manage the `Context` internally.
//!
//! So when using this type to switch to AlternateScreen. It will use the `Context` from the type `Crossterm` for the `AlternateSceen`.
//!
//!      For example:
//!
//!      // create crossterm instance.
//!      let crossterm = ::crossterm::Crossterm::new();
//!
//!      let mut cursor = crossterm.cursor();
//!      cursor.goto(10,10);
//!
//!      // create instance of Alternatescreen by the given refrence to crossterm, this wil also switch to it.
//!      let mut screen = crossterm::AlternateScreen::from(&crossterm);
//!
//!      // this cursor will be moving on the alternate screen since the current screen is the alternate screen.
//!      let mut cursor = crossterm.cursor();
//!      cursor.goto(10,10);
//!
//! As you can see because we are using `Crossterm` we won't have to bother about the `Context`.

use shared::functions;
use state::commands::*;
use Context;

use std::io::{self, Write};
use std::rc::Rc;
use std::convert::From;

pub struct AlternateScreen {
    context: Rc<Context>,
    command_id: u16,
}

impl AlternateScreen {
    /// Get the alternate screen from the context.
    /// By calling this method the current screen will be changed to the alternate screen.
    /// And you get back an handle for that screen.
    pub fn from(context: Rc<Context>) -> Self {
        let command_id = get_to_alternate_screen_command(context.clone());

        let screen = AlternateScreen {
            context: context.clone(),
            command_id: command_id,
        };
        screen.to_alternate();
        return screen;
    }

    /// Change the current screen to the mainscreen.
    pub fn to_main(&self) {
        let mut mutex = &self.context.state_manager;
        {
            let mut state_manager = mutex.lock().unwrap();

            let mut mx = &state_manager.get(self.command_id);
            {
                let mut command = mx.lock().unwrap();
                command.undo();
            }
        }
    }

    /// Change the current screen to alternate screen.
    pub fn to_alternate(&self) {
        let mut mutex = &self.context.state_manager;
        {
            let mut state_manager = mutex.lock().unwrap();

            let mut mx = &state_manager.get(self.command_id);
            {
                let mut command = mx.lock().unwrap();
                command.execute();
            }
        }
    }
}

impl Write for AlternateScreen {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut screen = self.context.screen_manager.lock().unwrap();
        {
            screen.write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut screen = self.context.screen_manager.lock().unwrap();
        {
            screen.flush()
        }
    }
}

impl Drop for AlternateScreen {
    fn drop(&mut self) {
        use CommandManager;
        CommandManager::undo(self.context.clone(), self.command_id);
    }
}

use super::super::shared::crossterm::Crossterm;

impl From<Crossterm> for AlternateScreen
{
    fn from(crossterm: Crossterm) -> Self {
        let command_id = get_to_alternate_screen_command(crossterm.context());

        let screen = AlternateScreen {
            context: crossterm.context(),
            command_id: command_id,
        };
        screen.to_alternate();
        return screen;
    }
}

// Get the alternate screen command to enable and disable alternate screen based on the current platform
fn get_to_alternate_screen_command(context: Rc<Context>) -> u16 {
    #[cfg(target_os = "windows")]
    let command_id = functions::get_module::<u16>(
        win_commands::ToAlternateScreenBufferCommand::new(context.clone()),
        shared_commands::ToAlternateScreenBufferCommand::new(context.clone()),
    ).unwrap();

    #[cfg(not(target_os = "windows"))]
    let command_id = shared_commands::ToAlternateScreenBufferCommand::new(context.clone());

    return command_id;
}
