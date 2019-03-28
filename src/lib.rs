#![cfg(target_os = "macos")]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate plygui_api;

#[macro_use]
pub extern crate objc;
pub use dispatch;
pub use cocoa;
pub use core_foundation;
pub use core_graphics;

#[macro_use]
pub mod common;

mod application;
mod button;
mod frame;
mod layout_linear;
mod splitted;
mod window;
mod text;
mod message;
mod image;
mod tray;

default_markup_register_members!();
default_pub_use!();
