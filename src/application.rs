use super::common::*;
use super::*;

use plygui_api::controls;
use plygui_api::types;

use cocoa::appkit::{NSApplication, NSApplicationActivationPolicy};
use dispatch::Queue;

use std::any::TypeId;

lazy_static! {
    static ref WINDOW_CLASS: RefClass = unsafe { register_window_class("PlyguiApplication", BASE_CLASS, |_| {}) };
    static ref DELEGATE: RefClass = unsafe { register_delegate() };
}
const BASE_CLASS: &str = "NSApplication";
const DEFAULT_FRAME_SLEEP_MS: u32 = 10;

pub type Application = AApplication<CocoaApplication>;

pub struct CocoaApplication {
    app: cocoa_id,
    delegate: *mut Object,
    name: String,
    sleep: u32,
}

impl HasNativeIdInner for CocoaApplication {
    type Id = common::CocoaId;

    fn native_id(&self) -> Self::Id {
        self.app.into()
    }
}

impl CocoaApplication {
    pub(crate) fn set_app_menu(&mut self, menu: cocoa_id) {
        /*unsafe {
            let menuu: cocoa_id = msg_send![self.app, mainMenu];
            menuu.itemAtIndex_(1).setSubmenu_(menu);
        }*/
        unsafe {
            let () = msg_send![self.app, setMainMenu: menu];
        }
    }
    fn apply_execution_policy(&mut self) {
        unsafe {
            let base = &mut cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)).unwrap().base;
            if base.windows.len() < 1 && base.trays.len() > 0 {
                self.app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory);
            } else {
                self.app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular);
            }
        }
    }
    pub fn maybe_exit(&mut self) -> bool {
        unsafe {
            let base = &mut cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)).unwrap().base;
            if base.windows.len() < 1 && base.trays.len() < 1 {
                self.app.setActivationPolicy_(NSApplicationActivationPolicy::NSApplicationActivationPolicyProhibited);
                let app: cocoa_id = msg_send![WINDOW_CLASS.0, sharedApplication];
                let () = msg_send![app, terminate:self.app];
                true
            } else {
                false
            }
        }
    }
}

impl<O: controls::Application> NewApplicationInner<O> for CocoaApplication {
    fn with_uninit_params(u: &mut mem::MaybeUninit<O>, name: &str) -> Self {
        let a = CocoaApplication {
            app: unsafe { msg_send![WINDOW_CLASS.0, sharedApplication] },
            delegate: unsafe { msg_send!(DELEGATE.0, new) },
            name: name.to_owned(),
            sleep: DEFAULT_FRAME_SLEEP_MS,
        };
        unsafe {
            let selfptr = u as *mut _ as *mut c_void;
            (&mut *a.app).set_ivar(IVAR, selfptr);
            (&mut *a.delegate).set_ivar(IVAR, selfptr);
            let () = msg_send![a.app, setDelegate: a.delegate];
        }
        a
    }
}

impl ApplicationInner for CocoaApplication {
    fn with_name<S: AsRef<str>>(name: S) -> Box<dyn controls::Application> {
        let mut b: Box<mem::MaybeUninit<Application>> = Box::new_uninit();
        let ab = AApplication::with_inner(
            <Self as NewApplicationInner<Application>>::with_uninit_params(b.as_mut(), name.as_ref()),
        );
        let selfptr = b.as_ptr() as usize;
        let a = unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        };
        Queue::main().exec_async(move || application_frame_runner(selfptr));
        a
    }
    fn add_root(&mut self, m: Box<dyn controls::Closeable>) -> &mut dyn controls::Member {
        let base = unsafe { &mut cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)).unwrap().base };
        
        let is_window = m.as_any().type_id() == TypeId::of::<crate::window::Window>();
        let is_tray = m.as_any().type_id() == TypeId::of::<crate::tray::Tray>();
        
        if is_window {
            let i = base.windows.len();
            base.windows.push(m.into_any().downcast::<crate::window::Window>().unwrap());
            self.apply_execution_policy();
            return base.windows[i].as_mut().as_member_mut();
        }
        
        if is_tray {
            let i = base.trays.len();
            base.trays.push(m.into_any().downcast::<crate::tray::Tray>().unwrap());
            self.apply_execution_policy();
            return base.trays[i].as_mut().as_member_mut();
        }
        
        panic!("Unsupported Closeable: {:?}", m.as_any().type_id());
    }
    fn close_root(&mut self, arg: types::FindBy, skip_callbacks: bool) -> bool {
        let base = unsafe { &mut cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)).unwrap().base }; 
        match arg {
            types::FindBy::Id(id) => {
                (0..base.windows.len()).into_iter().find(|i| if base.windows[*i].id() == id 
                    && base.windows[*i].as_any_mut().downcast_mut::<crate::window::Window>().unwrap().inner_mut().inner_mut().inner_mut().inner_mut().close(skip_callbacks) {
                        base.windows.remove(*i);
                        self.apply_execution_policy();
                        self.maybe_exit();
                        true
                    } else {
                        false
                }).is_some()
                || 
                (0..base.trays.len()).into_iter().find(|i| if base.trays[*i].id() == id 
                    && base.trays[*i].as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap().inner_mut().close(skip_callbacks) {
                        base.trays.remove(*i);
                        self.apply_execution_policy();
                        self.maybe_exit();
                        true
                    } else {
                        false
                }).is_some()
            }
            types::FindBy::Tag(tag) => {
                (0..base.windows.len()).into_iter().find(|i| if base.windows[*i].tag().is_some() && base.windows[*i].tag().unwrap() == Cow::Borrowed(tag.into()) 
                    && base.windows[*i].as_any_mut().downcast_mut::<crate::window::Window>().unwrap().inner_mut().inner_mut().inner_mut().inner_mut().close(skip_callbacks) {
                        base.windows.remove(*i);
                        self.apply_execution_policy();
                        self.maybe_exit();
                        true
                    } else {
                        false
                }).is_some()
                || 
                (0..base.trays.len()).into_iter().find(|i| if base.trays[*i].tag().is_some() && base.trays[*i].tag().unwrap() == Cow::Borrowed(tag.into()) 
                    && base.trays[*i].as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap().inner_mut().close(skip_callbacks) {
                        base.trays.remove(*i);
                        self.apply_execution_policy();
                        self.maybe_exit();
                        true
                    } else {
                        false
                }).is_some()
            }
        }
    }
    fn name(&self) -> ::std::borrow::Cow<'_, str> {
        ::std::borrow::Cow::Borrowed(self.name.as_ref())
    }
    fn frame_sleep(&self) -> u32 {
        self.sleep
    }
    fn set_frame_sleep(&mut self, value: u32) {
        self.sleep = value;
    }     
    fn start(&mut self) {
        unsafe { self.app.run() };
    }
    fn find_member_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn Member> {
        let base = unsafe { &mut cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)).unwrap().base }; 
        
        for window in base.windows.as_mut_slice() {
             match arg {
                types::FindBy::Id(id) => {
                    if window.id() == id {
                        return Some(window.as_member_mut());
                    }
                }
                types::FindBy::Tag(tag) => {
                    if let Some(mytag) = window.tag() {
                        if tag == mytag {
                            return Some(window.as_member_mut());
                        }
                    }
                }
            }
            let found = controls::Container::find_control_mut(window.as_mut(), arg.clone()).map(|control| control.as_member_mut());
            if found.is_some() {
                return found;
            }
        }
        for tray in base.trays.as_mut_slice() {
        	match arg {
                types::FindBy::Id(ref id) => {
                    if tray.id() == *id {
                        return Some(tray.as_member_mut());
                    }
                }
                types::FindBy::Tag(tag) => {
                    if let Some(mytag) = tray.tag() {
                        if tag == mytag {
                            return Some(tray.as_member_mut());
                        }
                    }
                }
            }
        }
        None
    }
    fn find_member<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn Member> {
        let base = unsafe { &mut cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)).unwrap().base }; 
        
        for window in base.windows.as_slice() {
            match arg {
                types::FindBy::Id(id) => {
                    if window.id() == id {
                        return Some(window.as_member());
                    }
                }
                types::FindBy::Tag(tag) => {
                    if let Some(mytag) = window.tag() {
                        if tag == mytag {
                            return Some(window.as_member());
                        }
                    }
                }
            }
            let found = controls::Container::find_control(window.as_ref(), arg.clone()).map(|control| control.as_member());
            if found.is_some() {
                return found;
            }
        }
        for tray in base.trays.as_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if tray.id() == *id {
                        return Some(tray.as_member());
                    }
                }
                types::FindBy::Tag(tag) => {
                    if let Some(mytag) = tray.tag() {
                        if tag == mytag {
                            return Some(tray.as_member());
                        }
                    }
                }
            }
        }
        None
    }
    #[allow(unused_comparisons)] // WAT?
    fn exit(&mut self) {
        let base = unsafe { &mut cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)).unwrap().base }; 
        
        for mut window in base.windows.drain(..) {
            window.as_any_mut().downcast_mut::<crate::window::Window>().unwrap().inner_mut().inner_mut().inner_mut().inner_mut().close(true);
        }
        for mut tray in base.trays.drain(..) {
            tray.as_any_mut().downcast_mut::<crate::tray::Tray>().unwrap().inner_mut().close(true);
        }

        self.maybe_exit();
    }
    fn roots<'a>(&'a self) -> Box<dyn Iterator<Item = &'a (dyn controls::Member)> + 'a> {
        unsafe { cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)) }.unwrap().roots()
    }
    fn roots_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut (dyn controls::Member)> + 'a> {
        unsafe { cast_cocoa_id_to_ptr(self.app).map(|ptr| &mut *(ptr as *mut _ as *mut Application)) }.unwrap().roots_mut()
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
    decl.add_method(sel!(applicationDidFinishLaunching:), application_did_finish_launching as extern "C" fn(&Object, Sel, cocoa_id));
    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}

extern "C" fn application_did_finish_launching(this: &Object, _sel: Sel, _notification: cocoa_id) {
    if let Some(app) = unsafe { from_cocoa_id_mut(this as *const _ as *mut Object) } {
        unsafe {
            let () = msg_send![app.inner_mut().app, activateIgnoringOtherApps: YES];
        }
        app.inner_mut().apply_execution_policy();
    }
}

extern "C" fn application_should_terminate_after_last_window_closed(_: &Object, _: Sel, _: cocoa_id) -> BOOL {
    NO
}

fn application_frame_runner(selfptr: usize) {
    let app: &mut Application = unsafe { mem::transmute(selfptr) };
    let mut frame_callbacks;
    {
        frame_callbacks = 0;
        let b = &mut app.base;
        while frame_callbacks < defaults::MAX_FRAME_CALLBACKS {
            match b.queue().try_recv() {
                Ok(mut cmd) => {
                    let app2: &mut Application = unsafe { mem::transmute(selfptr) };
                    if (cmd.as_mut())(app2) {
                        let _ = b.sender().send(cmd);
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
    unsafe {
        let () = msg_send![class!(NSThread), sleepForTimeInterval:(1.0 as f64 / app.inner().sleep as f64)];
    }
    Queue::main().exec_async(move || application_frame_runner(selfptr));
}

unsafe fn from_cocoa_id_mut<'a>(id: cocoa_id) -> Option<&'a mut Application> {
    cast_cocoa_id_to_ptr(id).map(|ptr| mem::transmute(ptr as *mut _ as *mut ::std::os::raw::c_void))
}
