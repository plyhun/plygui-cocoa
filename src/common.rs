use super::*;

use std::{ptr, mem, str};
use std::os::raw::c_void;
use std::slice;

use self::cocoa::base::{class, id as cocoa_id};
use self::cocoa::foundation::{NSString, NSRect, NSRange};
use self::cocoa::appkit::NSView;
use objc::runtime::{Class, Object, Ivar, YES, NO, class_copyIvarList};

use plygui_api::{development, ids, layout, types, callbacks};

pub struct RefClass(pub *const Class);
unsafe impl Sync for RefClass {}

pub const IVAR: &str = "plyguiIvar";

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
	pub fn with_params(invalidate: unsafe fn(this: &mut CocoaControlBase), functions: development::UiMemberFunctions) -> CocoaControlBase {
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
        	control: ptr::null_mut(),
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
            parent_cocoa_id(self.control, false).map(|id| cast_cocoa_id(id).unwrap())
        }
    }
    pub fn parent_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        unsafe {
            parent_cocoa_id(self.control, false).map(|id| cast_cocoa_id_mut(id).unwrap())
        }
    }
    pub fn root(&self) -> Option<&types::UiMemberBase> {
        unsafe {
            parent_cocoa_id(self.control, true).map(|id| cast_cocoa_id(id).unwrap())
        }
    }
    pub fn root_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        unsafe {
            parent_cocoa_id(self.control, true).map(|id| cast_cocoa_id_mut(id).unwrap())
        }
    }
}

/*pub unsafe fn cast_uicontrol_to_cocoa_mut(input: &mut Box<UiControl>) -> &mut CocoaControl {
    use std::ops::DerefMut;
    match input.role_mut() {
        UiRoleMut::Button(_) => {
            let a: &mut Box<button::Button> = mem::transmute(input);
            a.deref_mut()
        }
        UiRoleMut::LinearLayout(_) => {
            let a: &mut Box<layout_linear::LinearLayout> = mem::transmute(input);
            a.deref_mut()
        }
        UiRoleMut::Window(_) => {
            panic!("Window as a container child is impossible!");
        }
        _ => {
            unimplemented!();
        }
    }
}

pub unsafe fn cast_cocoa_id_to_cocoa<'a>(id: cocoa_id) -> Option<&'a mut CocoaControl> {
	if id.is_null() {
        return None;
    }
    let dlg = id.delegate();
    let mut ivar_count = 0;
    let ivars = class_copyIvarList(msg_send![dlg, class], &mut ivar_count);
    let ivar: &Ivar = mem::transmute(*ivars);
    let id_: &Object = mem::transmute(dlg);
    let saved: *mut c_void = *id_.get_ivar(ivar.name());
    match ivar.name() {
        super::layout_linear::IVAR => {
            let ll: &mut LinearLayout = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
            Some(ll)
        }
        super::button::IVAR => {
            let w: &mut Button = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
            Some(w)
        }
        _ => None,
    }
}*/
pub unsafe fn parent_cocoa_id(id: cocoa_id, is_root: bool) -> Option<cocoa_id> {
	let id_: cocoa_id = if is_root { msg_send![id, window] } else { msg_send![id, superview] };
    if id_.is_null() || id_ == id {
        None
    } else {
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
    let mut ivar_count = 0;
    let class = msg_send![id, class];
    let ivars = class_copyIvarList(class, &mut ivar_count);
    let ivars: &[&Ivar] = slice::from_raw_parts_mut(ivars as *mut _, ivar_count as usize);
    let id_: &Object = mem::transmute(
    	if ivars.iter().any(|ivar| ivar.name() == IVAR) { id } else { parent_cocoa_id(id, true).unwrap() }
    );
    Some(*id_.get_ivar(IVAR))
}

/*pub unsafe fn cast_cocoa_id_to_uicontainer<'a>(id: cocoa_id) -> Option<&'a mut UiContainer> {
	if id.is_null() {
        return None;
    }
    let dlg = id.delegate();
    let mut ivar_count = 0;
    let ivars = class_copyIvarList(msg_send![dlg, class], &mut ivar_count);
    let ivar: &Ivar = mem::transmute(*ivars);
    let id_: &Object = mem::transmute(dlg);
    let saved: *mut c_void = *id_.get_ivar(ivar.name());
    match ivar.name() {
        super::layout_linear::IVAR => {
            let ll: &mut LinearLayout = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
            Some(ll)
        },
        super::window::IVAR => {
            let w: &mut Window = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
            Some(w)
        }
        _ => None,
    }
}*/

/*pub unsafe fn cast_uicontrol_to_cocoa(input: &Box<UiControl>) -> &CocoaControl {
    use std::ops::Deref;
    match input.role() {
        UiRole::Button(_) => {
            let a: &Box<button::Button> = mem::transmute(input);
            a.deref()
        }
        UiRole::LinearLayout(_) => {
            let a: &Box<layout_linear::LinearLayout> = mem::transmute(input);
            a.deref()
        }
        UiRole::Window(_) => {
            panic!("Window as a container child is impossible!");
        }
        _ =>{
	        unimplemented!();
        }
    }
}*/

pub unsafe fn measure_string(text: &str) -> (u16, u16) {
    let title = NSString::alloc(cocoa::base::nil).init_str(text);

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
#[macro_export]
macro_rules! impl_is_control {
	($typ: ty) => {
		unsafe fn is_control(this: &::plygui_api::development::UiMemberCommon) -> Option<&::plygui_api::development::UiControlCommon> {
			Some(&::plygui_api::utils::base_to_impl::<$typ>(this).base.control_base)
		}
		unsafe fn is_control_mut(this: &mut ::plygui_api::development::UiMemberCommon) -> Option<&mut ::plygui_api::development::UiControlCommon> {
			Some(&mut ::plygui_api::utils::base_to_impl_mut::<$typ>(this).base.control_base)
		}
	}
}
#[macro_export]
macro_rules! impl_size {
	($typ: ty) => {
		unsafe fn size(this: &::plygui_api::development::UiMemberCommon) -> (u16, u16) {
			::plygui_api::utils::base_to_impl::<$typ>(this).size()
		}
	}
}
#[macro_export]
macro_rules! impl_member_id {
	($mem: expr) => {
		unsafe fn member_id(_: &::plygui_api::development::UiMemberCommon) -> &'static str {
			$mem
		}
	}
}
#[macro_export]
macro_rules! impl_measure {
	($typ: ty) => {
		unsafe fn measure(&mut UiMemberBase, w: u16, h: u16) -> (u16, u16, bool) {
			::plygui_api::utils::base_to_impl::<$typ>(this).measure(w, h)
		}
	}
}
#[macro_export]
macro_rules! impl_draw {
	($typ: ty) => {
		unsafe fn draw(&mut UiMemberBase, coords: Option<(i32, i32)>) {
			::plygui_api::utils::base_to_impl::<$typ>(this).draw(coords)
		}
	}
}