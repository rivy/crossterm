//! This module contains the commands that can be used for unix systems.

use super::IStateCommand;
use kernel::unix_kernel::terminal;
use termios::{tcsetattr, Termios, CREAD, ECHO, ICANON, TCSAFLUSH};
use {CommandManager, Context, StateManager};

const FD_STDIN: ::std::os::unix::io::RawFd = 1;

use std::rc::Rc;
use std::sync::Mutex;

/// This command is used for switching to NoncanonicalMode.
#[derive(Copy, Clone)]
pub struct NoncanonicalModeCommand {
    key: u16,
}

impl NoncanonicalModeCommand {
    pub fn new(state_manager: &Mutex<StateManager>) -> u16 {
        let mut state = state_manager.lock().unwrap();
        {
            let key = state.get_changes_count();
            let command = NoncanonicalModeCommand { key: key };

            state.register_change(Box::from(command), key);
            key
        }
    }
}

impl IStateCommand for NoncanonicalModeCommand {
    fn execute(&mut self) -> bool {
        // Set noncanonical mode
        if let Ok(orig) = Termios::from_fd(FD_STDIN) {
            let mut noncan = orig.clone();
            noncan.c_lflag &= !ICANON;
            noncan.c_lflag &= !ECHO;
            noncan.c_lflag &= !CREAD;
            match tcsetattr(FD_STDIN, TCSAFLUSH, &noncan) {
                Ok(_) => return true,
                Err(_) => return false,
            };
        } else {
            return false;
        }
    }

    fn undo(&mut self) -> bool {
        // Disable noncanonical mode
        if let Ok(orig) = Termios::from_fd(FD_STDIN) {
            let mut noncan = orig.clone();
            noncan.c_lflag &= ICANON;
            noncan.c_lflag &= ECHO;
            noncan.c_lflag &= CREAD;

            match tcsetattr(FD_STDIN, TCSAFLUSH, &noncan) {
                Ok(_) => return true,
                Err(_) => return false,
            };
        } else {
            return false;
        }
    }
}

/// This command is used for enabling and disabling raw mode for the terminal.
pub struct EnableRawModeCommand {
    original_mode: Option<Box<Termios>>,
    command_id: u16,
}

impl EnableRawModeCommand {
    pub fn new(state_manager: &Mutex<StateManager>) -> u16 {
        let mut state = state_manager.lock().unwrap();
        {
            let key = state.get_changes_count();
            let command = EnableRawModeCommand {
                original_mode: None,
                command_id: key,
            };

            state.register_change(Box::from(command), key);
            key
        }
    }
}

impl IStateCommand for EnableRawModeCommand {
    fn execute(&mut self) -> bool {
        let original_mode = terminal::get_terminal_mode();

        if let Ok(original_mode) = original_mode {
            self.original_mode = Some(Box::from(original_mode));
            let mut new_mode = original_mode;
            terminal::make_raw(&mut new_mode);
            terminal::set_terminal_mode(&new_mode);
            true
        } else {
            return false;
        }
    }

    fn undo(&mut self) -> bool {
        if let Some(ref original_mode) = self.original_mode {
            let result = terminal::set_terminal_mode(&original_mode);

            match result {
                Ok(()) => true,
                Err(_) => false,
            }
        } else {
            return false;
        }
    }
}
