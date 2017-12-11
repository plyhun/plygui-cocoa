#![cfg(target_os="macos")]

#[macro_use]
extern crate lazy_static;

extern crate plygui_api;

#[macro_use]
extern crate objc;
extern crate cocoa;
extern crate core_foundation;

#[macro_use]
pub mod common;

mod application;
mod window;
mod button;
mod layout_linear;
//mod layout_relative;

pub type NativeId = winapi::HWND;

pub use self::application::Application;
pub use self::window::Window;
pub use self::button::Button;
pub use self::layout_linear::LinearLayout;
//pub use self::layout_relative::RelativeLayout;