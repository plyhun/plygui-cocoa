use super::common::*;

use self::cocoa::appkit::{NSApplication, NSApplicationActivationPolicyRegular, NSEventMask};
use self::cocoa::foundation::NSDefaultRunLoopMode;

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    pub static NSApplicationWillFinishLaunchingNotification: cocoa_id;
    pub static NSApplicationDidFinishLaunchingNotification: cocoa_id;
}

lazy_static! {
    static ref WINDOW_CLASS: RefClass = unsafe { register_window_class("PlyguiApplication", BASE_CLASS, |decl| {
        decl.add_method(sel!(run), application_run as extern "C" fn(&mut Object, Sel));
        decl.add_method(sel!(terminate:), application_terminate as extern "C" fn(&mut Object, Sel, cocoa_id));
        decl.add_method(sel!(stop:), application_stop as extern "C" fn(&mut Object, Sel, cocoa_id));
        decl.add_method(sel!(isRunning:), application_is_running as extern "C" fn(&mut Object, Sel, cocoa_id) -> BOOL);
    }) };
    static ref DELEGATE: RefClass = unsafe { register_delegate() };
}
const BASE_CLASS: &str = "NSApplication";

pub type Application = plygui_api::development::Application<CocoaApplication>;

pub struct CocoaApplication {
    app: cocoa_id,
    delegate: *mut Object,
    name: String,
    running: bool,
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
                    running: false,
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
    fn find_member_by_id(&self, id: ids::Id) -> Option<&controls::Member> {
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
        application_should_running_after_last_window_closed as extern "C" fn(&Object, Sel, cocoa_id) -> BOOL,
    );
    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}

extern "C" fn application_should_running_after_last_window_closed(_: &Object, _: Sel, _: cocoa_id) -> BOOL {
    YES
}
extern "C" fn application_run(this: &mut Object, _: Sel) {
    let mut frame_callbacks;
    unsafe {
        let app: &mut Application = cast_cocoa_id_to_ptr(this).map(|ptr| mem::transmute(ptr)).unwrap();
        app.as_inner_mut().running = true;
        
        let default_center: cocoa_id = msg_send![class!(NSNotificationCenter), defaultCenter];
        let () = msg_send![default_center, postNotificationName:NSApplicationWillFinishLaunchingNotification object:(this as cocoa_id)];
        let () = msg_send![default_center, postNotificationName:NSApplicationDidFinishLaunchingNotification object:(this as cocoa_id)];

        while app.as_inner_mut().running {
            let distant_future: cocoa_id = msg_send![class!(NSDate), distantFuture];
            let event: cocoa_id = msg_send![this,
					nextEventMatchingMask:NSEventMask::NSAnyEventMask
					untilDate: distant_future
					inMode:NSDefaultRunLoopMode
					dequeue:YES];
            if !event.is_null() {
                let () = msg_send![this, sendEvent: event];
            }
            let () = msg_send![this, updateWindows];
            {
                for window_id in app.as_inner_mut().windows.as_mut_slice() {
                    let window: &mut window::Window = cast_cocoa_id_to_ptr(*window_id).map(|ptr| mem::transmute(ptr)).unwrap();
                    frame_callbacks = 0;
                    while frame_callbacks < defaults::MAX_FRAME_CALLBACKS {
                        let window = window.as_inner_mut().as_inner_mut().base_mut();
                        match window.queue().try_recv() {
                            Ok(mut cmd) => {
                                if (cmd.as_mut())(cast_cocoa_id_to_ptr(*window_id).map(|ptr| mem::transmute::<*mut c_void, &mut window::Window>(ptr)).unwrap()) {
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
        } {}
        println!("Exit");
    }
}
extern "C" fn application_is_running(this: &mut Object, _: Sel, _: cocoa_id) -> BOOL {
    let app: &mut Application = unsafe { cast_cocoa_id_to_ptr(this).map(|ptr| mem::transmute(ptr)).unwrap() };
    if app.as_inner_mut().running { YES } else { NO }
}
extern "C" fn application_terminate(this: &mut Object, _: Sel, _: cocoa_id) {
    println!("terminate");
    {
        let app: &mut Application = unsafe { cast_cocoa_id_to_ptr(this).map(|ptr| mem::transmute(ptr)).unwrap() };
        app.as_inner_mut().running = false;
    }
    unsafe { let () = msg_send![super(this, Class::get(BASE_CLASS).unwrap()), terminate]; }
}
extern "C" fn application_stop(this: &mut Object, _: Sel, _: cocoa_id) {
    println!("stop");
    {
        let app: &mut Application = unsafe { cast_cocoa_id_to_ptr(this).map(|ptr| mem::transmute(ptr)).unwrap() };
        app.as_inner_mut().running = false;
    }
    unsafe { let () = msg_send![super(this, Class::get(BASE_CLASS).unwrap()), stop]; }
}

