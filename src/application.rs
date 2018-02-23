use super::*;
use super::common::*;

use self::cocoa::appkit::{NSApplication, NSApplicationActivationPolicyRegular};
use self::cocoa::base::{id};
use objc::runtime::{Class, Object, Sel, BOOL, YES};
use objc::declare::ClassDecl;

use std::os::raw::c_void;

use plygui_api::members::MEMBER_ID_APPLICATION;
use plygui_api::traits::{UiWindow, UiApplication, UiMember};
use plygui_api::types::WindowStartSize;
use plygui_api::ids::Id;

lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
	static ref DELEGATE: RefClass = unsafe { register_delegate() };
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
            (&mut *app.app).set_ivar(common::IVAR,
                                     app.as_mut() as *mut _ as *mut ::std::os::raw::c_void);
            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(IVAR,
                                      app.as_mut() as *mut _ as *mut ::std::os::raw::c_void);
            msg_send!(app.app, setDelegate:delegate);

			app.app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
            app
        }
    }
}

impl UiApplication for Application {
    fn new_window(&mut self, title: &str, size: WindowStartSize, has_menu: bool) -> Box<UiWindow> {
        let w = Window::new(title, size, has_menu);
        self.windows.push(w.window);
        w
    }
    fn name<'a>(&'a self) -> ::std::borrow::Cow<'a, str> {
        ::std::borrow::Cow::Borrowed(self.name.as_ref())
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

unsafe fn register_delegate() -> RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiApplicationDelegate", superclass).unwrap();

    decl.add_method(sel!(applicationShouldTerminateAfterLastWindowClosed:),
                    application_should_terminate_after_last_window_closed as extern "C" fn(&Object, Sel, id) -> BOOL);
    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}

unsafe fn register_window_class() -> RefClass {
    let superclass = Class::get("NSApplication").unwrap();
    let mut decl = ClassDecl::new(MEMBER_ID_APPLICATION, superclass).unwrap();

    decl.add_ivar::<*mut c_void>(common::IVAR);

    RefClass(decl.register())
}

extern "C" fn application_should_terminate_after_last_window_closed(_: &Object, _: Sel, _: id) -> BOOL {
    YES
}
