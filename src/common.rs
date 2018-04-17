use super::*;

use std::{ptr, mem, str};
use std::os::raw::c_void;
use std::slice;

use self::cocoa::base::{class, id as cocoa_id};
use self::cocoa::foundation::{NSString, NSRect, NSSize, NSPoint, NSRange};
use self::cocoa::appkit::NSView;
use objc::runtime::{Class, Ivar, YES, NO, class_copyIvarList};
use objc::declare::ClassDecl;

use plygui_api::{development, ids, layout, types, callbacks};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RefClass(pub *const Class);
unsafe impl Sync for RefClass {}

pub const IVAR: &str = "plyguiIvar";
pub const IVAR_PARENT: &str = "plyguiIvarParent";

#[repr(C)]
pub struct CocoaControlBase {
    pub control_base: development::UiControlCommon, 
    
    pub control: cocoa_id,
    pub coords: Option<(i32, i32)>,
    pub measured_size: (u16, u16),
    pub h_resize: Option<callbacks::Resize>,
    
    invalidate: unsafe fn(this: &mut CocoaControlBase),
}

impl CocoaControlBase {
	pub fn with_params(class: RefClass, invalidate: unsafe fn(this: &mut CocoaControlBase), functions: development::UiMemberFunctions) -> CocoaControlBase {
		CocoaControlBase {
        	control_base: development::UiControlCommon {
	        	member_base: development::UiMemberCommon::with_params(types::Visibility::Visible, functions),
		        layout: layout::Attributes {
		            width: layout::Size::MatchParent,
					height: layout::Size::WrapContent,
					gravity: layout::gravity::CENTER_HORIZONTAL | layout::gravity::TOP,
					..Default::default()
	            },
        	},
        	control: unsafe {
        		let rect = NSRect::new(NSPoint::new(0f64, 0f64), NSSize::new(0f64, 0f64));

		        let mut control: cocoa_id = msg_send![class.0, alloc];
		        control = msg_send![control, initWithFrame: rect];
				control = msg_send![control, autorelease];
	        	control
	        },
            h_resize: None,
            coords: None,
            measured_size: (0, 0),
            
            invalidate: invalidate
        }
	}
	pub fn frame(&self) -> NSRect {
		unsafe { msg_send![self.control, frame] }
	}
	pub fn invalidate(&mut self) {
		unsafe { (self.invalidate)(self) }
	}
    pub unsafe fn on_removed_from_container(&mut self) {
        self.control.removeFromSuperview();
        let () = msg_send![self.control, dealloc];
        self.control = ptr::null_mut();
    }   
    pub fn set_visibility(&mut self, visibility: types::Visibility) {
        if self.control_base.member_base.visibility != visibility {
            self.control_base.member_base.visibility = visibility;
            unsafe {
                let () = match self.control_base.member_base.visibility {
                    types::Visibility::Visible => {
                        msg_send![self.control, setHidden: NO]
                    }
                    _ => {
                        msg_send![self.control, setHidden: YES]
                    }
                };
            }
            self.invalidate();
        }
    }
    pub fn visibility(&self) -> types::Visibility {
        self.control_base.member_base.visibility
    }
    pub fn id(&self) -> ids::Id {
        self.control_base.member_base.id
    }
    pub fn parent_cocoa_id(&self) -> Option<cocoa_id> {
    	unsafe {
    		parent_cocoa_id(self.control, false)
    	}
    }
    pub fn parent(&self) -> Option<&types::UiMemberBase> {
        unsafe {
            parent_cocoa_id(self.control, false).and_then(|id| cast_cocoa_id(id))
        }
    }
    pub fn parent_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        unsafe {
            parent_cocoa_id(self.control, false).and_then(|id| cast_cocoa_id_mut(id))
        }
    }
    pub fn root(&self) -> Option<&types::UiMemberBase> {
        unsafe {
            parent_cocoa_id(self.control, true).and_then(|id| cast_cocoa_id(id))
        }
    }
    pub fn root_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        unsafe {
            parent_cocoa_id(self.control, true).and_then(|id| cast_cocoa_id_mut(id))
        }
    }
}

pub unsafe fn parent_cocoa_id(id: cocoa_id, is_root: bool) -> Option<cocoa_id> {
	let id_: cocoa_id = if is_root { 
        msg_send![id, window] 
    } else if let Some(parent) = has_cocoa_id_ivar(id, IVAR_PARENT) { 
        parent as cocoa_id
    } else {
    	msg_send![id, superview] 
    };
    if id_.is_null() || id_ == id {
        None
    } else {
    	let clas: *mut Class = msg_send![id, class];
	    let classp: *mut Class = msg_send![id_, class];
	    println!("parent of {} is {}", (&*clas).name(), (&*classp).name());
	    Some(id_)
    }
}
pub unsafe fn cast_cocoa_id_mut<'a, T>(id: cocoa_id) -> Option<&'a mut T> where T: Sized {
	cast_cocoa_id_to_ptr(id).map(|ptr| mem::transmute(ptr as *mut _ as *mut ::std::os::raw::c_void))
}
pub unsafe fn cast_cocoa_id<'a, T>(id: cocoa_id) -> Option<&'a T> where T: Sized {
	cast_cocoa_id_to_ptr(id).map(|ptr| mem::transmute(ptr as *mut _ as *const ::std::os::raw::c_void))
}
pub unsafe fn cast_cocoa_id_to_ptr<'a>(id: cocoa_id) -> Option<*mut c_void> {
	if id.is_null() {
        return None;
    }    
	
    if let Some(parent) = has_cocoa_id_ivar(id, IVAR) {
	    Some(parent)
    } else { 
		parent_cocoa_id(id, true).and_then(|id| cast_cocoa_id_to_ptr(id))
    }
}

pub unsafe fn has_cocoa_id_ivar(id: cocoa_id, ivar: &str) -> Option<*mut c_void> {
	if id.is_null() {
        return None;
    }    
    let mut ivar_count = 0;
    let class = msg_send![id, class];
    let ivars = class_copyIvarList(class, &mut ivar_count);
    let ivars: &[&Ivar] = slice::from_raw_parts_mut(ivars as *mut _, ivar_count as usize);
    
    if ivars.iter().any(|va| va.name() == ivar) {
    	let ivar: *mut c_void = *(&mut *id).get_ivar(ivar);
		if !ivar.is_null() { 
			return Some(ivar) 
		}
    } 
    None
}

pub unsafe fn measure_string(text: &str) -> (u16, u16) {
	let title = NSString::alloc(cocoa::base::nil).init_str(text);
    measure_nsstring(title)
}

pub unsafe fn measure_nsstring(title: cocoa_id) -> (u16, u16) {
    let text_storage: cocoa_id = msg_send![class("NSTextStorage"), alloc];
    let text_storage: cocoa_id = msg_send![text_storage, initWithString: title];
    let layout_manager: cocoa_id = msg_send![class("NSLayoutManager"), alloc];
    let layout_manager: cocoa_id = msg_send![layout_manager, init];
    let text_container: cocoa_id = msg_send![class("NSTextContainer"), alloc];
    let text_container: cocoa_id = msg_send![text_container, init];

    let () = msg_send![layout_manager, addTextContainer: text_container];
    let () = msg_send![text_container, release];
    let () = msg_send![text_storage, addLayoutManager: layout_manager];
    let () = msg_send![layout_manager, release];

    let num = msg_send![layout_manager, numberOfGlyphs];
    let range = NSRange::new(0, num);

    let string_rect: NSRect = msg_send![layout_manager, boundingRectForGlyphRange:range inTextContainer:text_container];
    (string_rect.size.width as u16, string_rect.size.height as u16)
}

pub unsafe fn register_window_class<F>(name: &str, base: &str, mut f: F) -> RefClass where F: FnMut(&mut ClassDecl) {
    let superclass = Class::get(base).unwrap();
    let mut decl = ClassDecl::new(name, superclass).unwrap();

    decl.add_ivar::<*mut c_void>(IVAR);
    decl.add_ivar::<*mut c_void>(IVAR_PARENT);
    
    f(&mut decl);

    common::RefClass(decl.register())
}

#[macro_export]
macro_rules! impl_invalidate {
	($typ: ty) => {
		unsafe fn invalidate_impl(this: &mut common::CocoaControlBase) {
			use plygui_api::development::UiDrawable;
			use plygui_api::members::MEMBER_ID_WINDOW;
			use objc::runtime::YES;
			
			let parent_hwnd = this.parent_cocoa_id();	
			if let Some(parent_hwnd) = parent_hwnd {
				let mparent = common::cast_cocoa_id_mut::<plygui_api::development::UiMemberCommon>(parent_hwnd).unwrap();
				let (pw, ph) = mparent.size();
				let this: &mut $typ = mem::transmute(this);
				
				let (_,_,changed) = this.measure(pw, ph);
				
				if changed {
					if mparent.is_control().is_some() {
						common::cast_cocoa_id_mut::<common::CocoaControlBase>(parent_hwnd).unwrap().invalidate();
					} else if mparent.member_id() == MEMBER_ID_WINDOW {
						this.draw(None);	
						let () = msg_send![parent_hwnd, setNeedsDisplay:YES];
					} else {
						panic!("Parent member {} is unsupported, neither a control, nor a window", mparent.member_id());
					}
				} else {
					this.draw(None);	
				}
		    }
		}
	}
}
