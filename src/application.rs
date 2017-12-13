use super::*;
use super::common::*;

use self::cocoa::appkit::{NSApplication, NSApplicationActivationPolicyRegular};
use self::cocoa::base::{class, id};
use objc::runtime::{Class, Object};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;

use plygui_api::traits::{UiWindow, UiApplication};
use plygui_api::types::WindowStartSize;

lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}

pub struct Application {
    app: id,
    name: String,
}

impl Application {
    pub fn with_name(name: &str) -> Box<Application> {
        unsafe {
            let mut app = Box::new(Application {
                                       app: msg_send![WINDOW_CLASS.0, sharedApplication],
                                       name: name.into(),
                                   });
            (&mut *app.app).set_ivar("plyguiApplication",
                                     app.as_mut() as *mut _ as *mut ::std::os::raw::c_void);
            app.app
                .setActivationPolicy_(NSApplicationActivationPolicyRegular);
            app
        }
    }

    pub(crate) fn on_window_closed(&mut self) {
        println!("App closed");
        //unsafe { msg_send![class("NSApp"), terminate:self]; }
    }
}

impl UiApplication for Application {
    fn new_window(&mut self, title: &str, size: WindowStartSize, has_menu: bool) -> Box<UiWindow> {
        Window::new(title, size, has_menu)
    }
    fn name(&self) -> &str {
        self.name.as_ref()
    }
    fn start(&mut self) {
        unsafe { self.app.run() };
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe {
            msg_send![self.app, dealloc];
        }
    }
}

unsafe fn register_window_class() -> RefClass {
    let superclass = Class::get("NSApplication").unwrap();
    let mut decl = ClassDecl::new("PlyguiApplication", superclass).unwrap();

    decl.add_ivar::<*mut c_void>("plyguiApplication");

    RefClass(decl.register())
}
