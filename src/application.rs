use super::*;
use super::common::*;

use self::cocoa::appkit::{NSApplication, NSApplicationActivationPolicyRegular};
use self::cocoa::base::{id};
use objc::runtime::{Class};
use objc::declare::ClassDecl;

use std::os::raw::c_void;

use plygui_api::traits::{UiWindow, UiApplication, UiMember};
use plygui_api::types::WindowStartSize;
use plygui_api::ids::Id;

lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}

pub struct Application {
    app: id,
    name: String,
    windows: Vec<id>,
}

impl Application {
    pub fn with_name(name: &str) -> Box<Application> {
        unsafe {
            let mut app = Box::new(Application {
                                       app: msg_send![WINDOW_CLASS.0, sharedApplication],
                                       name: name.into(),
                                       windows: Vec::with_capacity(1),
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
        let w = Window::new(title, size, has_menu);
        self.windows.push(w.window);
        w
    }
    fn name(&self) -> &str {
        self.name.as_ref()
    }
    fn start(&mut self) {
        unsafe { self.app.run() };
    }
    fn find_member_by_id_mut(&mut self, id: Id) -> Option<&mut UiMember> {
    	use plygui_api::traits::UiContainer;
    	
    	for window in self.windows.as_mut_slice() {
    		if let Some(window) = unsafe { common::cast_cocoa_id_mut::<Window>(*window) } {
    			if window.as_base().id() == id {
	    			return Some(window);
	    		} else {
	    			return window.find_control_by_id_mut(id).map(|control| control.as_member_mut());
	    		}
    		}
    	}
    	None
    }
    fn find_member_by_id(&self, id: Id) -> Option<&UiMember> {
    	use plygui_api::traits::UiContainer;
    	
    	for window in self.windows.as_slice() {
    		if let Some(window) = unsafe { common::cast_cocoa_id_mut::<Window>(*window) } {
    			if window.as_base().id() == id {
	    			return Some(window);
	    		} else {
	    			return window.find_control_by_id(id).map(|control| control.as_member());
	    		}
    		}
    	}
    	None
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
