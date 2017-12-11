use super::*;
use super::common::*;

use self::cocoa::appkit::{NSWindow, NSRunningApplication, NSClosableWindowMask, NSResizableWindowMask, NSMiniaturizableWindowMask, NSTitledWindowMask, NSBackingStoreBuffered};
use self::cocoa::foundation::{NSString, NSAutoreleasePool, NSRect, NSSize, NSPoint};
use self::cocoa::base::{id, nil};
use objc::runtime::{Class, Object, Sel, BOOL, YES, NO};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;

use {development, ids, Id, UiRole, UiRoleMut, UiWindow, UiControl, UiMember, UiContainer, UiMultiContainer, Visibility};

pub const IVAR: &str = development::CLASS_ID_WINDOW;
lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}

pub struct Window {
    id: Id,
    window: id,
    container: id,
    visibility: Visibility,

    child: Option<Box<UiControl>>,
    h_resize: Option<Box<FnMut(&mut UiMember, u16, u16)>>,
}

impl Window {
    pub(crate) fn new(
                      title: &str,
                      width: u16,
                      height: u16,
                      has_menu: bool)
                      -> Box<Window> {
        use self::cocoa::appkit::NSView;

        unsafe {
            let window = NSWindow::alloc(nil)
                .initWithContentRect_styleMask_backing_defer_(NSRect::new(NSPoint::new(0.0, 0.0),
                                                                          NSSize::new(width as f64, height as f64)),
                                                              NSClosableWindowMask | NSResizableWindowMask | NSMiniaturizableWindowMask | NSTitledWindowMask,
                                                              NSBackingStoreBuffered,
                                                              NO)
                .autorelease();
            window.cascadeTopLeftFromPoint_(NSPoint::new(20., 20.));
            window.center();
            let title = NSString::alloc(cocoa::base::nil).init_str(title);
            window.setTitle_(title);
            window.makeKeyAndOrderFront_(cocoa::base::nil);
            let current_app = cocoa::appkit::NSRunningApplication::currentApplication(cocoa::base::nil);
            current_app.activateWithOptions_(cocoa::appkit::NSApplicationActivateIgnoringOtherApps);

            let view = NSView::alloc(nil)
                .initWithFrame_(NSRect::new(NSPoint::new(0.0, 0.0),
                                            NSSize::new(width as f64, height as f64)))
                .autorelease();
            window.setContentView_(view);

            let mut window = Box::new(Window {
							            id: ids::next(),
                                          window: window,
                                          container: view,
                                          visibility: Visibility::Visible,

                                          child: None,
                                          h_resize: None,
                                      });

            let delegate: *mut Object = msg_send!(WINDOW_CLASS.0, new);
            (&mut *delegate).set_ivar(IVAR,
                                      window.as_mut() as *mut _ as *mut ::std::os::raw::c_void);
            window.window.setDelegate_(delegate);

            window
        }
    }
}

impl UiWindow for Window {}

impl UiContainer for Window {
    fn set_child(&mut self, mut child: Option<Box<UiControl>>) -> Option<Box<UiControl>> {
        use self::cocoa::appkit::NSView;

        unsafe {
            let mut old = self.child.take();
            if let Some(old) = old.as_mut() {
                let mut wc = common::cast_uicontrol_to_cocoa_mut(old);
                wc.on_removed_from_container(self);
            }
            if let Some(new) = child.as_mut() {
            	let (_, h) = self.size();
                let mut wc = common::cast_uicontrol_to_cocoa_mut(new);
                wc.on_added_to_container(self, 0, 0); //TODO padding
                self.container.addSubview_(wc.as_base().control); 
            }
            self.child = child;

            old
        }
    }
    fn child(&self) -> Option<&UiControl> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut UiControl> {
        //self.child.as_mut().map(|c|c.as_mut()) // WTF ??
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
    fn find_control_by_id_mut(&mut self, id_: Id) -> Option<&mut UiControl> {
        /*if self.id() == id_ {
			return Some(self);
		} else*/
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id_);
            }
        }
        None
    }
    fn find_control_by_id(&self, id_: Id) -> Option<&UiControl> {
        /*if self.id() == id_ {
			return Some(self);
		} else*/
        if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id_);
            }
        }
        None
    }
    fn is_multi_mut(&mut self) -> Option<&mut UiMultiContainer> {
        None
    }
    fn is_multi(&self) -> Option<&UiMultiContainer> {
        None
    }
}

impl UiMember for Window {
    fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
        unsafe {
            if Visibility::Visible == visibility {
                msg_send![self.window, setIsVisible: YES];
            } else {
                msg_send![self.window, setIsVisible: NO];
            }
        }
    }
    fn visibility(&self) -> Visibility {
        self.visibility
    }
    fn size(&self) -> (u16, u16) {
        unsafe {
            let size = self.window.contentView().frame().size;
            (size.width as u16, size.height as u16)
        }
    }
    fn on_resize(&mut self, handler: Option<Box<FnMut(&mut UiMember, u16, u16)>>) {
        self.h_resize = handler;
    }

    fn role<'a>(&'a self) -> UiRole<'a> {
        UiRole::Window(self)
    }
    fn role_mut<'a>(&'a mut self) -> UiRoleMut<'a> {
        UiRoleMut::Window(self)
    }
    fn id(&self) -> Id {
        self.id
    }
    fn native_id(&self) -> NativeId {
    	self.window
    }
    
    fn is_control_mut(&mut self) -> Option<&mut UiControl> {
    	None
    }
    fn is_control(&self) -> Option<&UiControl> {
    	None
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            msg_send![self.container, dealloc];
            msg_send![self.window, dealloc];
        }
    }
}

unsafe impl CocoaContainer for Window {
    unsafe fn cocoa_id(&self) -> id {
        self.window
    }
}

unsafe fn register_window_class() -> RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiWindowDelegate", superclass).unwrap();

    decl.add_method(sel!(applicationShouldTerminateAfterLastWindowClosed:),
                    application_should_terminate_after_last_window_closed as extern "C" fn(&Object, Sel, id) -> BOOL);
    decl.add_method(sel!(windowShouldClose:),
                    window_should_close as extern "C" fn(&Object, Sel, id) -> BOOL);
    decl.add_method(sel!(windowDidResize:),
                    window_did_resize as extern "C" fn(&Object, Sel, id));
    decl.add_method(sel!(windowDidChangeScreen:),
                    window_did_change_screen as extern "C" fn(&Object, Sel, id));
    //decl.add_method(sel!(windowWillClose:), window_will_close as extern "C" fn(&Object, Sel, id));

    //decl.add_method(sel!(windowDidBecomeKey:), window_did_become_key as extern "C" fn(&Object, Sel, id));
    //decl.add_method(sel!(windowDidResignKey:), window_did_resign_key as extern "C" fn(&Object, Sel, id));

    decl.add_ivar::<*mut c_void>(IVAR);
    //decl.add_ivar::<*mut c_void>("plyguiApplication");

    RefClass(decl.register())
}

extern "C" fn application_should_terminate_after_last_window_closed(_: &Object, _: Sel, _: id) -> BOOL {
    YES
}

extern "C" fn window_did_resize(this: &Object, _: Sel, _: id) {
    unsafe {
        let saved: *mut c_void = *this.get_ivar(IVAR);
        let window: &mut Window = mem::transmute(saved.clone());
        let size = window.window.contentView().frame().size;

        if let Some(ref mut child) = window.child {
            let (_, h, _) = child.measure(size.width as u16, size.height as u16);
            child.draw(Some((0, size.height as i32 - h as i32))); //TODO padding
        }
        if let Some(ref mut cb) = window.h_resize {
            let w2: &mut Window = mem::transmute(saved);
            (cb)(w2, size.width as u16, size.height as u16);
        }
    }
}

extern "C" fn window_did_change_screen(this: &Object, _: Sel, _: id) {
    unsafe {
        let saved: *mut c_void = *this.get_ivar(IVAR);
        let window: &mut Window = mem::transmute(saved.clone());
        if let Some(ref mut cb) = window.h_resize {
            let size = window.window.contentView().frame().size;
            let w2: &mut Window = mem::transmute(saved);
            (cb)(w2, size.width as u16, size.height as u16);
        }
    }
}
extern "C" fn window_should_close(_: &Object, _: Sel, _: id) -> BOOL {
    YES
}
