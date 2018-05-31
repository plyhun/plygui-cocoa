use super::*;

use self::cocoa::appkit::{NSApplication, NSApplicationActivationPolicyRegular};
use self::cocoa::base::{id};
use objc::runtime::{Class, Object, Sel, BOOL, YES};
use objc::declare::ClassDecl;

use plygui_api::development::{HasInner, WindowInner, MemberInner};

use std::os::raw::c_void;

use plygui_api::{development, controls, types, ids};

lazy_static! {
	static ref WINDOW_CLASS: common::RefClass = unsafe { common::register_window_class("PlyguiApplication", "NSApplication", |_| {}) };
	static ref DELEGATE: common::RefClass = unsafe { register_delegate() };
}

pub type Application = development::Application<CocoaApplication>;

pub struct CocoaApplication {
    app: id,
    delegate: *mut Object,
    name: String,
    windows: Vec<id>,
}

impl development::ApplicationInner for CocoaApplication {
	fn with_name(name: &str) -> Box<controls::Application> {
		unsafe {
            let mut app = Box::new(development::Application::with_inner(CocoaApplication {
                                       app: msg_send![WINDOW_CLASS.0, sharedApplication],
                                       delegate: msg_send!(DELEGATE.0, new),
                                       name: name.to_owned(),
                                       windows: Vec::with_capacity(1),
                                   }, ()));
            let selfptr = app.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
            (&mut *app.as_inner_mut().app).set_ivar(common::IVAR, selfptr);
            (&mut *app.as_inner_mut().delegate).set_ivar(common::IVAR, selfptr);
            let () = msg_send![app.as_inner_mut().app, setDelegate: app.as_inner_mut().delegate];
            let () = msg_send![app.as_inner_mut().app, setActivationPolicy:NSApplicationActivationPolicyRegular];
            app
        }
	}
	fn new_window(&mut self, title: &str, size: types::WindowStartSize, menu: types::WindowMenu) -> Box<controls::Window> {
		let w = window::CocoaWindow::with_params(title, size, menu);
        unsafe { self.windows.push(w.as_single_container().as_container().as_member().as_any().downcast_ref::<window::Window>().unwrap().as_inner().native_id().into()); }
        w
	}
    fn name(&self) -> ::std::borrow::Cow<str> {
    	::std::borrow::Cow::Borrowed(self.name.as_ref())
    }
    fn start(&mut self) {
    	unsafe { self.app.run() };
    }
    fn find_member_by_id_mut(&mut self, id: ids::Id) -> Option<&mut controls::Member> {
    	use plygui_api::controls::{Member, Container};
    	
    	for window in self.windows.as_mut_slice() {
    		if let Some(window) = unsafe { common::member_from_cocoa_id_mut::<Window>(*window) } {
    			if window.id() == id {
	    			return Some(window);
	    		} else {
	    			return window.find_control_by_id_mut(id).map(|control| control.as_member_mut());
	    		}
    		}
    	}
    	None
    }
    fn find_member_by_id(&self, id: ids::Id) -> Option<&controls::Member> {
    	use plygui_api::controls::{Member, Container};
    	
    	for window in self.windows.as_slice() {
    		if let Some(window) = unsafe { common::member_from_cocoa_id::<Window>(*window) } {
    			if window.id() == id {
	    			return Some(window);
	    		} else {
	    			return window.find_control_by_id(id).map(|control| control.as_member());
	    		}
    		}
    	}
    	None
    }
}

impl Drop for CocoaApplication {
    fn drop(&mut self) {
        unsafe {
            let () = msg_send![self.app, dealloc];
            let () = msg_send![self.delegate, dealloc];
        }
    }
}

unsafe fn register_delegate() -> common::RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiApplicationDelegate", superclass).unwrap();

    decl.add_method(sel!(applicationShouldTerminateAfterLastWindowClosed:),
                    application_should_terminate_after_last_window_closed as extern "C" fn(&Object, Sel, id) -> BOOL);
    decl.add_ivar::<*mut c_void>(common::IVAR);

    common::RefClass(decl.register())
}

extern "C" fn application_should_terminate_after_last_window_closed(_: &Object, _: Sel, _: id) -> BOOL {
    YES
}
