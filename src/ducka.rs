use crate::window::{WindowSystem};

// Manage entire app
pub struct Duck {
    window_s: WindowSystem, // Holds the window system
    command_l: u16, // Holds the row to be used to enter command inputs
    status_l: u16, // Holds the row to be used to output
}