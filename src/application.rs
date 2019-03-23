use super::common::*;

use self::cocoa::appkit::{NSApplication, NSApplicationActivationPolicy};
use self::dispatch::Queue;

lazy_static! {
    static ref WINDOW_CLASS: RefClass = unsafe { register_window_class("PlyguiApplication", BASE_CLASS, |_| {}) };
    static ref DELEGATE: RefClass = unsafe { register_delegate() };
}
const BASE_CLASS: &str = "NSApplication";

pub type Application = plygui_api::development::Application<CocoaApplication>;

pub struct CocoaApplication {
    app: cocoa_id,
    delegate: *mut Object,
    name: String,

    pub(crate) windows: Vec<cocoa_id>,
    pub(crate) trays: Vec<cocoa_id>,
}

impl HasNativeIdInner for CocoaApplication {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.app.into()
    }
}

impl CocoaApplication {
    pub(crate) fn remove_window(&mut self, id: cocoa_id) {
        self.windows.retain(|i| *i == id);
        self.apply_execution_policy();
        self.try_exit();
    }
    pub(crate) fn remove_tray(&mut self, id: cocoa_id) {
        self.trays.retain(|i| *i == id);
        self.apply_execution_policy();
        self.try_exit();
    }
    fn apply_execution_policy(&mut self) {
        if self.windows.len() < 1 && self.trays.len() > 0 {
            unsafe {
                self.app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyProhibited);
            }
        } else {
            unsafe {
                self.app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);
                let () = msg_send![self.app, activateIgnoringOtherApps: YES];
            }
        }
    }
    fn try_exit(&mut self) -> bool {
        if self.windows.len() < 1 && self.trays.len() < 1 {
            unsafe {
                let app: cocoa_id = msg_send![WINDOW_CLASS.0, sharedApplication];
                let () = msg_send![app, terminate:self.app];
            }
            true
        } else {
            false
        }
    }
}

impl ApplicationInner for CocoaApplication {
    fn get() -> Box<Application> {
        unsafe {
            let mut app = Box::new(plygui_api::development::Application::with_inner(
                CocoaApplication {
                    app: msg_send![WINDOW_CLASS.0, sharedApplication],
                    delegate: msg_send!(DELEGATE.0, new),
                    name: String::new(), // name.to_owned(), // TODO later
                    windows: Vec::with_capacity(1),
                    trays: vec![],
                },
                (),
            ));
            let selfptr = app.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
            (&mut *app.as_inner_mut().app).set_ivar(IVAR, selfptr);
            (&mut *app.as_inner_mut().delegate).set_ivar(IVAR, selfptr);
            let () = msg_send![app.as_inner_mut().app, setDelegate: app.as_inner_mut().delegate];
            let () = msg_send![app.as_inner_mut().app, setActivationPolicy: NSApplicationActivationPolicy::NSApplicationActivationPolicyProhibited];

            let selfptr = selfptr as usize;
            Queue::main().r#async(move || application_frame_runner(selfptr));

            app
        }
    }
    fn new_window(&mut self, title: &str, size: types::WindowStartSize, menu: types::Menu) -> Box<dyn controls::Window> {
        use plygui_api::controls::HasNativeId;

        let w = window::CocoaWindow::with_params(title, size, menu);
        unsafe {
            self.windows.push(w.native_id() as cocoa_id);
        }
        self.apply_execution_policy();
        w
    }
    fn new_tray(&mut self, title: &str, menu: types::Menu) -> Box<dyn controls::Tray> {
        use plygui_api::controls::HasNativeId;

        let tray = tray::CocoaTray::with_params(title, menu);
        unsafe {
            self.trays.push(tray.native_id() as cocoa_id);
        }
        self.apply_execution_policy();
        tray
    }
    fn name(&self) -> ::std::borrow::Cow<'_, str> {
        ::std::borrow::Cow::Borrowed(self.name.as_ref())
    }
    fn start(&mut self) {
        unsafe { self.app.run() };
    }
    fn find_member_by_id_mut(&mut self, id: ids::Id) -> Option<&mut dyn controls::Member> {
        use plygui_api::controls::{Container, Member};

        for window in self.windows.as_mut_slice() {
            if let Some(window) = unsafe { member_from_cocoa_id_mut::<super::window::Window>(*window) } {
                if window.id() == id {
                    return Some(window);
                } else {
                    return window.find_control_by_id_mut(id).map(|control| control.as_member_mut());
                }
            }
        }
        None
    }
    fn find_member_by_id(&self, id: ids::Id) -> Option<&dyn controls::Member> {
        use plygui_api::controls::{Container, Member};

        for window in self.windows.as_slice() {
            if let Some(window) = unsafe { member_from_cocoa_id::<super::window::Window>(*window) } {
                if window.id() == id {
                    return Some(window);
                } else {
                    return window.find_control_by_id(id).map(|control| control.as_member());
                }
            }
        }
        None
    }
    #[allow(unused_comparisons)] // WAT?
    fn exit(&mut self, skip_on_close: bool) -> bool {
        use crate::plygui_api::controls::Closeable;

        let mut n = self.windows.len();
        let mut i = n - 1;
        while i >= 0 {
            let window = &self.windows[i];
            if let Some(window) = unsafe { member_from_cocoa_id_mut::<super::window::Window>(*window) } {
                if !window.close(skip_on_close) {
                    return false;
                }
            }
            i -= 1;
        }

        n = self.trays.len();
        i = n - 1;
        while i >= 0 {
            let tray = &self.trays[i];
            if let Some(tray) = unsafe { member_from_cocoa_id_mut::<super::tray::Tray>(*tray) } {
                if !tray.close(skip_on_close) {
                    return false;
                }
            }
            i -= 1;
        }

        self.try_exit()
    }
}

impl Drop for CocoaApplication {
    fn drop(&mut self) {
        if self.exit(true) {
            unsafe {
                let () = msg_send![self.app, dealloc];
                let () = msg_send![self.delegate, dealloc];
            }
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
    decl.add_method(sel!(applicationDidFinishLaunching:), application_did_finish_launching as extern "C" fn(&Object, Sel, cocoa_id));
    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}

extern "C" fn application_did_finish_launching(this: &Object, _sel: Sel, _notification: cocoa_id) {
    if let Some(app) = unsafe { from_cocoa_id_mut(this as *const _ as *mut Object) } {
        app.as_inner_mut().apply_execution_policy();
    }
}

extern "C" fn application_should_terminate_after_last_window_closed(_: &Object, _: Sel, _: cocoa_id) -> BOOL {
    NO
}

fn application_frame_runner(selfptr: usize) {
    let app: &mut Application = unsafe { mem::transmute(selfptr) };
    let mut frame_callbacks;
    {
        for window_id in app.as_inner_mut().windows.as_mut_slice() {
            let window: &mut window::Window = unsafe { cast_cocoa_id_to_ptr(*window_id).map(|ptr| mem::transmute(ptr)).unwrap() };
            frame_callbacks = 0;
            while frame_callbacks < defaults::MAX_FRAME_CALLBACKS {
                let window = window.as_inner_mut().as_inner_mut().base_mut();
                match window.queue().try_recv() {
                    Ok(mut cmd) => {
                        if (cmd.as_mut())(unsafe { cast_cocoa_id_to_ptr(*window_id).map(|ptr| mem::transmute::<*mut c_void, &mut window::Window>(ptr)).unwrap() }) {
                            let _ = window.sender().send(cmd);
                        }
                        frame_callbacks += 1;
                    }
                    Err(e) => match e {
                        mpsc::TryRecvError::Empty => break,
                        mpsc::TryRecvError::Disconnected => unreachable!(),
                    },
                }
            }
        }
    }
    Queue::main().r#async(move || application_frame_runner(selfptr));
}

unsafe fn from_cocoa_id_mut<'a>(id: cocoa_id) -> Option<&'a mut Application> {
    cast_cocoa_id_to_ptr(id).map(|ptr| mem::transmute(ptr as *mut _ as *mut ::std::os::raw::c_void))
}
