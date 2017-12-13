use super::*;
use super::common::*;

use self::cocoa::appkit::{NSWindow, NSRunningApplication, NSClosableWindowMask, NSResizableWindowMask, NSMiniaturizableWindowMask, NSTitledWindowMask, NSBackingStoreBuffered};
use self::cocoa::foundation::{NSString, NSAutoreleasePool, NSRect, NSSize, NSPoint};
use self::cocoa::base::{id, nil};
use objc::runtime::{Class, Object, Sel, BOOL, YES, NO};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;

use plygui_api::{development, ids, types};
use plygui_api::traits::{UiControl, UiWindow, UiSingleContainer, UiMember, UiContainer};
use plygui_api::members::MEMBER_ID_WINDOW;

pub const IVAR: &str = MEMBER_ID_WINDOW;
lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}

pub struct Window {
	base: development::UiMemberBase,
	
    window: id,
    container: id,
    
    child: Option<Box<UiControl>>,
    h_resize: Option<Box<FnMut(&mut UiMember, u16, u16)>>,
}

impl Window {
    pub(crate) fn new(
                      title: &str,
                      start_size: types::WindowStartSize,
                      has_menu: bool)
                      -> Box<Window> {
        use self::cocoa::appkit::NSView;

        unsafe {
        	let rect = NSRect::new(NSPoint::new(0.0, 0.0),
                                                                          match start_size {
	                	types::WindowStartSize::Exact(width, height) => NSSize::new(width as f64, height as f64),
	                	types::WindowStartSize::Fullscreen => unimplemented!(),
                	});
            let window = NSWindow::alloc(nil)
                .initWithContentRect_styleMask_backing_defer_(rect,
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
                .initWithFrame_(rect)
                .autorelease();
            window.setContentView_(view);

            let mut window = Box::new(Window {
							            base: development::UiMemberBase::with_params(
								            types::Visibility::Visible,
						                    development::UiMemberFunctions {
						                        fn_member_id: member_id,
						                        fn_is_control: is_control,
						                        fn_is_control_mut: is_control_mut,
						                        fn_size: size,
						                    }
							            ),
                                          window: window,
                                          container: view,
                                          
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

impl UiSingleContainer for Window {
	fn set_child(&mut self, mut child: Option<Box<UiControl>>) -> Option<Box<UiControl>> {
        use self::cocoa::appkit::NSView;

        unsafe {
            let mut old = self.child.take();
            if let Some(old) = old.as_mut() {
                old.on_removed_from_container(self);
            }
            if let Some(new) = child.as_mut() {
            	let (_, h) = self.size();
                new.on_added_to_container(self, 0, 0); //TODO padding
                self.container.addSubview_(new.native_id() as id); 
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
}

impl UiContainer for Window {
    fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut UiControl> {
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
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&UiControl> {
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
    fn is_single_mut(&mut self) -> Option<&mut UiSingleContainer> {
        Some(self)
    }
    fn is_single(&self) -> Option<&UiSingleContainer> {
        Some(self)
    }
}

impl UiMember for Window {
    fn set_visibility(&mut self, visibility: types::Visibility) {
        self.base.visibility = visibility;
        unsafe {
            if types::Visibility::Visible == visibility {
                msg_send![self.window, setIsVisible: YES];
            } else {
                msg_send![self.window, setIsVisible: NO];
            }
        }
    }
    fn visibility(&self) -> types::Visibility {
        self.base.visibility
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
	fn member_id(&self) -> &'static str {
        self.base.member_id()
    }
    
    fn id(&self) -> ids::Id {
        self.base.id
    }
    unsafe fn native_id(&self) -> usize {
    	self.window as usize
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

unsafe fn is_control(_: &development::UiMemberBase) -> Option<&development::UiControlBase> {
    None
}
unsafe fn is_control_mut(_: &mut development::UiMemberBase) -> Option<&mut development::UiControlBase> {
    None
}
impl_size!(Window);
impl_member_id!(MEMBER_ID_WINDOW);

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
