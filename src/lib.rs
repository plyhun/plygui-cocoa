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

pub type NativeId = cocoa::base::id;

pub use self::application::Application;
pub use self::window::Window;
pub use self::button::Button;
pub use self::layout_linear::LinearLayout;
//pub use self::layout_relative::RelativeLayout;

#[cfg(feature = "markup")]
pub fn register_members(registry: &mut plygui_api::markup::MarkupRegistry) {
	//registry.insert(plygui_api::members::MEMBER_ID_BUTTON.into(), button::spawn);
	//registry.insert(plygui_api::members::MEMBER_ID_LAYOUT_LINEAR.into(), layout_linear::spawn);
	registry.insert(plygui_api::markup::MEMBER_TYPE_BUTTON.into(), button::spawn);
	registry.insert(plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT.into(), layout_linear::spawn);
}