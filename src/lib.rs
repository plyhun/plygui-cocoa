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

pub use plygui_api::external;

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

#[cfg(feature = "markup")]
pub fn register_members(registry: &mut plygui_api::markup::MarkupRegistry) {
    registry.register_member(plygui_api::markup::MEMBER_TYPE_BUTTON.into(), button::spawn).unwrap();
    registry.register_member(plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT.into(), layout_linear::spawn).unwrap();
    registry.register_member(plygui_api::markup::MEMBER_TYPE_FRAME.into(), frame::spawn).unwrap();
    registry.register_member(plygui_api::markup::MEMBER_TYPE_SPLITTED.into(), splitted::spawn).unwrap();
    registry.register_member(plygui_api::markup::MEMBER_TYPE_IMAGE.into(), image::spawn).unwrap();
}

pub mod prelude {
	pub use plygui_api::controls::*;
	pub use plygui_api::ids::*;
	pub use plygui_api::types::*;
	pub use plygui_api::callbacks;
	pub use plygui_api::layout;
	pub use plygui_api::utils; 
	
	pub mod imp {
		pub use crate::application::Application;
		pub use crate::window::Window;
		pub use crate::button::Button;
		pub use crate::layout_linear::LinearLayout;
		pub use crate::frame::Frame;
		pub use crate::splitted::Splitted;
		pub use crate::text::Text;
		pub use crate::message::Message;
		pub use crate::image::Image;
		pub use crate::tray::Tray;
	}
}
