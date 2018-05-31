use super::*;

use plygui_api::{development, ids, types, controls, layout};
use plygui_api::development::HasInner;

use self::cocoa::appkit::{NSWindow, NSWindowStyleMask, NSBackingStoreBuffered};
use self::cocoa::foundation::{NSString, NSAutoreleasePool, NSRect, NSSize, NSPoint};
use self::cocoa::base::{id, nil};
use objc::runtime::{Class, Object, Sel, BOOL, YES, NO};
use objc::declare::ClassDecl;

use std::os::raw::c_void;
use std::borrow::Cow;
use std::ffi::CString;

const BASE_CLASS: &str = "NSWindow";

lazy_static! {
	static ref WINDOW_CLASS: common::RefClass = unsafe { register_window_class() };
	static ref DELEGATE: common::RefClass = unsafe { register_delegate() };
}

pub type Window = development::Member<development::SingleContainer<CocoaWindow>>;

#[repr(C)]
pub struct CocoaWindow {
	pub(crate) window: id,
    pub(crate) container: id,
    
    gravity_horizontal: layout::Gravity,
    gravity_vertical: layout::Gravity,
    
    child: Option<Box<controls::Control>>,
}

impl CocoaWindow {
	fn size_inner(&self) -> (u16, u16) {
    	unsafe {
            let size = self.window.contentView().frame().size;
            (size.width as u16, size.height as u16)
        }
    }
	fn redraw(&mut self) {
    	let size = self.size_inner();
    	if let Some(ref mut child) = self.child {
        	child.measure(size.0, size.1);
            child.draw(Some((0, 0)));
        }            
    }
}

impl development::WindowInner for CocoaWindow {
	fn with_params(title: &str, window_size: types::WindowStartSize, menu: types::WindowMenu) -> Box<controls::Window> {
		use self::cocoa::appkit::NSView;

        unsafe {
        	let rect = NSRect::new(NSPoint::new(0.0, 0.0),
                    match window_size {
	                	types::WindowStartSize::Exact(width, height) => NSSize::new(width as f64, height as f64),
	                	types::WindowStartSize::Fullscreen => unimplemented!(),
                	});
        	let window: id = msg_send![WINDOW_CLASS.0, alloc];
            let window = window
                .initWithContentRect_styleMask_backing_defer_(rect,
                                                              NSWindowStyleMask::NSClosableWindowMask | NSWindowStyleMask::NSResizableWindowMask | NSWindowStyleMask::NSMiniaturizableWindowMask | NSWindowStyleMask::NSTitledWindowMask,
                                                              NSBackingStoreBuffered,
                                                              NO);
            let () = msg_send![window ,cascadeTopLeftFromPoint: NSPoint::new(20., 20.)];
            window.center();
            let title = NSString::alloc(cocoa::base::nil).init_str(title);
            let () = msg_send![window, setTitle: title];
            let () = msg_send![window, makeKeyAndOrderFront: cocoa::base::nil];
            let current_app = cocoa::appkit::NSRunningApplication::currentApplication(cocoa::base::nil);
            let () = msg_send![current_app, activateWithOptions: cocoa::appkit::NSApplicationActivateIgnoringOtherApps];

            let view = NSView::alloc(nil).initWithFrame_(rect);
            let () = msg_send![window, setContentView: view];

            let mut window = Box::new(development::Member::with_inner(development::SingleContainer::with_inner(CocoaWindow {
							            window: window,
                                          container: view,
                                          
                                          gravity_horizontal: Default::default(),
										    gravity_vertical: Default::default(),    
											child: None,
                                      }, ()),
            		development::MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
            ));

            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(common::IVAR,
                                      window.as_mut() as *mut _ as *mut ::std::os::raw::c_void);
            (&mut *window.as_inner_mut().as_inner_mut().window).set_ivar(common::IVAR,
                                      window.as_mut() as *mut _ as *mut ::std::os::raw::c_void);
            let () = msg_send![window.as_inner_mut().as_inner_mut().window, setDelegate: delegate];

            window
        }
	}
}

impl development::HasLabelInner for CocoaWindow {
	fn label(&self) -> ::std::borrow::Cow<str> {
		unsafe { 
			let title: id = msg_send![self.window, title];
			let title = msg_send![title, UTF8String];
			Cow::Owned(CString::from_raw(title).into_string().unwrap())
		}
	}
    fn set_label(&mut self, _: &mut development::MemberBase, label: &str) {
    	unsafe {
    		let label = NSString::alloc(cocoa::base::nil).init_str(label);
	        let () = msg_send![self.window, setTitle: label];
    	}
    }
}

impl development::SingleContainerInner for CocoaWindow {
	fn set_child(&mut self, _: &mut development::MemberBase, mut child: Option<Box<controls::Control>>) -> Option<Box<controls::Control>> {
		use plygui_api::controls::SingleContainer;
		use plygui_api::development::MemberInner;
		
		let mut old = self.child.take();
        if let Some(old) = old.as_mut() {
            let outer_self = unsafe { common::member_from_cocoa_id_mut::<Window>(self.window).unwrap() };
        	let outer_self = outer_self.as_single_container_mut().as_container_mut();
            old.on_removed_from_container(outer_self);
        }
        if let Some(new) = child.as_mut() {
        	let (_, _) = self.size();
	        unsafe { let () = msg_send![self.container, addSubview: new.native_id() as id]; }
            let outer_self = unsafe { common::member_from_cocoa_id_mut::<Window>(self.window).unwrap() };
        	let outer_self = outer_self.as_single_container_mut().as_container_mut();
            new.on_added_to_container(outer_self, 0, 0);   
            new.draw(Some((0, 0)));         
        }
        self.child = child;

        old
	}
    fn child(&self) -> Option<&controls::Control> {
    	self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut controls::Control> {
    	//self.child.as_mut().map(|c|c.as_mut()) // WTF ??
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
}

impl development::ContainerInner for CocoaWindow {
	fn find_control_by_id_mut(&mut self, id: ids::Id) -> Option<&mut controls::Control> {
		if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id);
            }
        }
        None
	}
    fn find_control_by_id(&self, id: ids::Id) -> Option<&controls::Control> {
    	if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id);
            }
        }
        None
    }
    
    fn gravity(&self) -> (layout::Gravity, layout::Gravity) {
    	(self.gravity_horizontal, self.gravity_vertical)
    }
    fn set_gravity(&mut self, _: &mut development::MemberBase, w: layout::Gravity, h: layout::Gravity) {
    	if self.gravity_horizontal != w || self.gravity_vertical != h {
    		self.gravity_horizontal = w;
    		self.gravity_vertical = h;
    		self.redraw();
    	}
    }
}

impl development::MemberInner for CocoaWindow {
	type Id = common::CocoaId;
	
    fn size(&self) -> (u16, u16) {
    	self.size_inner()
    }
    
    fn on_set_visibility(&mut self, base: &mut development::MemberBase) {
    	unsafe {
            let () = if types::Visibility::Visible == base.visibility {
                msg_send![self.window, setIsVisible: YES]
            } else {
                msg_send![self.window, setIsVisible: NO]
            };
        }
    }
    
    unsafe fn native_id(&self) -> Self::Id {
    	self.window.into()
    }
}

impl Drop for CocoaWindow {
    fn drop(&mut self) {
        unsafe {
            let () = msg_send![self.container, dealloc];
            let () = msg_send![self.window, dealloc];
        }
    }
}

unsafe fn register_window_class() -> common::RefClass {
    let superclass = Class::get(BASE_CLASS).unwrap();
    let mut decl = ClassDecl::new("PlyguiWindow", superclass).unwrap();

    decl.add_ivar::<*mut c_void>(common::IVAR);

    common::RefClass(decl.register())
}

unsafe fn register_delegate() -> common::RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiWindowDelegate", superclass).unwrap();

    decl.add_method(sel!(windowShouldClose:),
                    window_should_close as extern "C" fn(&Object, Sel, id) -> BOOL);
    decl.add_method(sel!(windowDidResize:),
                    window_did_resize as extern "C" fn(&mut Object, Sel, id));
    decl.add_method(sel!(windowDidChangeScreen:),
                    window_did_change_screen as extern "C" fn(&mut Object, Sel, id));
    //decl.add_method(sel!(windowWillClose:), window_will_close as extern "C" fn(&Object, Sel, id));

    //decl.add_method(sel!(windowDidBecomeKey:), window_did_become_key as extern "C" fn(&Object, Sel, id));
    //decl.add_method(sel!(windowDidResignKey:), window_did_resign_key as extern "C" fn(&Object, Sel, id));

    decl.add_ivar::<*mut c_void>(common::IVAR);
    //decl.add_ivar::<*mut c_void>("plyguiApplication");

    common::RefClass(decl.register())
}

fn window_redraw(this: &mut Object) {
	use plygui_api::controls::Member;
	
	unsafe {
		let window = common::member_from_cocoa_id_mut::<Window>(this).unwrap();
        let size = window.size();

        window.as_inner_mut().as_inner_mut().redraw();
        
        if let Some(ref mut cb) = window.base_mut().handler_resize {
            use plygui_api::controls::SingleContainer;
            
            let mut w2 = common::member_from_cocoa_id_mut::<Window>(this).unwrap();
            (cb.as_mut())(w2.as_single_container_mut().as_container_mut().as_member_mut(), size.0, size.1);
        }
    }
}

extern "C" fn window_did_resize(this: &mut Object, _: Sel, _: id) {
    window_redraw(this)
}

extern "C" fn window_did_change_screen(this: &mut Object, _: Sel, _: id) {
    window_redraw(this)
}
extern "C" fn window_should_close(_: &Object, _: Sel, _: id) -> BOOL {
    YES
}

impl_all_defaults!(Window);