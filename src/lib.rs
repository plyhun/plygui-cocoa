#![feature(new_uninit)]
#![cfg(target_os = "macos")]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate plygui_api;

#[macro_use]
pub extern crate objc;
pub use cocoa;
pub use core_foundation;
pub use core_graphics;
pub use dispatch;

#[macro_use]
pub mod common;

mod application;
mod button;
mod frame;
mod image;
mod layout_linear;
mod message;
mod splitted;
mod text;
mod tray;
mod window;
mod progress_bar;
mod list;

default_markup_register_members!();
default_pub_use!();
