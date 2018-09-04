use super::common::*;

use self::cocoa::appkit::{NSApplication, NSApplicationActivationPolicyRegular};

lazy_static! {
    static ref WINDOW_CLASS: RefClass = unsafe { register_window_class("PlyguiApplication", "NSApplication", |_| {}) };
    static ref DELEGATE: RefClass = unsafe { register_delegate() };
}

pub type Application = plygui_api::development::Application<CocoaApplication>;

pub struct CocoaApplication {
    app: cocoa_id,
    delegate: *mut Object,
    name: String,
    windows: Vec<cocoa_id>,
}

impl ApplicationInner for CocoaApplication {
    fn with_name(name: &str) -> Box<Application> {
        unsafe {
            let mut app = Box::new(plygui_api::development::Application::with_inner(
                CocoaApplication {
                    app: msg_send![WINDOW_CLASS.0, sharedApplication],
                    delegate: msg_send!(DELEGATE.0, new),
                    name: name.to_owned(),
                    windows: Vec::with_capacity(1),
                },
                (),
            ));
            let selfptr = app.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
            (&mut *app.as_inner_mut().app).set_ivar(IVAR, selfptr);
            (&mut *app.as_inner_mut().delegate).set_ivar(IVAR, selfptr);
            let () = msg_send![app.as_inner_mut().app, setDelegate: app.as_inner_mut().delegate];
            let () = msg_send![app.as_inner_mut().app, setActivationPolicy: NSApplicationActivationPolicyRegular];
            app
        }
    }
    fn new_window(&mut self, title: &str, size: types::WindowStartSize, menu: types::WindowMenu) -> Box<controls::Window> {
        use plygui_api::controls::Member;

        let w = window::CocoaWindow::with_params(title, size, menu);
        unsafe {
            self.windows.push(w.native_id() as cocoa_id);
        }
        w
    }
    fn name(&self) -> ::std::borrow::Cow<str> {
        ::std::borrow::Cow::Borrowed(self.name.as_ref())
    }
    fn start(&mut self) {
        unsafe { self.app.run() };
    }
    fn find_member_by_id_mut(&mut self, id: ids::Id) -> Option<&mut controls::Member> {
        use plygui_api::controls::{Container, Member};

        for window in self.windows.as_mut_slice() {
            if let Some(window) = unsafe { member_from_cocoa_id_mut::<Window>(*window) } {
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
        use plygui_api::controls::{Container, Member};

        for window in self.windows.as_slice() {
            if let Some(window) = unsafe { member_from_cocoa_id::<Window>(*window) } {
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

unsafe fn register_delegate() -> RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiApplicationDelegate", superclass).unwrap();

    decl.add_method(
        sel!(applicationShouldTerminateAfterLastWindowClosed:),
        application_should_terminate_after_last_window_closed as extern "C" fn(&Object, Sel, cocoa_id) -> BOOL,
    );
    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}

extern "C" fn application_should_terminate_after_last_window_closed(_: &Object, _: Sel, _: cocoa_id) -> BOOL {
    YES
}
