use super::*;

use std::{ptr, mem, str};
use std::os::raw::c_void;
use self::cocoa::base::{class, id as cocoa_id};
use self::cocoa::foundation::{NSString, NSRect, NSRange};
use self::cocoa::appkit::{NSWindow, NSView};
use objc::runtime::{Class, Object, Ivar, YES, NO, class_copyIvarList};

use plygui_api::{development, ids, layout, types, callbacks};
use plygui_api::traits::UiMember;

pub struct RefClass(pub *const Class);
unsafe impl Sync for RefClass {}

pub const IVAR: &str = "plyguiIvar";

/*pub unsafe trait CocoaControl: UiMember {
    unsafe fn on_added_to_container(&mut self, &CocoaContainer, x: u16, y: u16);
    unsafe fn on_removed_from_container(&mut self, &CocoaContainer);
    fn as_base(&self) -> &CocoaControlBase;
    fn as_base_mut(&mut self) -> &mut CocoaControlBase;
}*/

#[repr(C)]
pub struct CocoaControlBase {
    pub control_base: development::UiControlCommon, 
    
    pub control: cocoa_id,
    pub coords: Option<(i32, i32)>,
    pub measured_size: (u16, u16),
    pub h_resize: Option<callbacks::Resize>,
    
    //invalidate: unsafe fn(this: &mut WindowsControlBase),
}

impl CocoaControlBase {
	pub fn invalidate(&mut self) {}
	pub fn with_params(functions: development::UiMemberFunctions) -> CocoaControlBase {
		CocoaControlBase {
        	control_base: development::UiControlCommon {
	        	member_base: development::UiMemberCommon::with_params(types::Visibility::Visible, functions),
		        layout: development::layout::LayoutBase {
		            width: layout::Size::MatchParent,
					height: layout::Size::WrapContent,
					gravity: layout::gravity::CENTER_HORIZONTAL | layout::gravity::TOP,
					orientation: layout::Orientation::Vertical,
					alignment: layout::Alignment::None,
	            },
        	},
        	control: ptr::null_mut(),
            h_resize: None,
            coords: None,
            measured_size: (0, 0),
        }
	}
    pub unsafe fn on_removed_from_container(&mut self) {
        self.control.removeFromSuperview();
        msg_send![self.control, dealloc];
        self.control = ptr::null_mut();
    }   
    pub fn set_visibility(&mut self, visibility: types::Visibility) {
        if self.control_base.member_base.visibility != visibility {
            self.control_base.member_base.visibility = visibility;
            unsafe {
                match self.control_base.member_base.visibility {
                    types::Visibility::Visible => {
                        msg_send![self.control, setHidden: NO];
                    }
                    _ => {
                        msg_send![self.control, setHidden: YES];
                    }
                }
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
    		let id_: cocoa_id = msg_send![self.control, superview];
	        if id_.is_null() {
	            None
	        } else {
	        	Some(id_)
	        }
    	}
    }
    pub fn parent(&self) -> Option<&types::UiMemberBase> {
        unsafe {
            let id_: cocoa_id = msg_send![self.control, superview];
            if id_.is_null() {
                return None;
            }
            cast_cocoa_id(id_)
        }
    }
    pub fn parent_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        unsafe {
            let id_: cocoa_id = msg_send![self.control, superview];
            if id_.is_null() {
                return None;
            }
            cast_cocoa_id_mut(id_)
        }
    }
    pub fn root(&self) -> Option<&types::UiMemberBase> {
        unsafe {
            let w: cocoa_id = msg_send![self.control, window];
            if w.is_null() {
                return None;
            }
            cast_cocoa_id(w.delegate())
        }
    }
    pub fn root_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        unsafe {
            let w: cocoa_id = msg_send![self.control, window];
            cast_cocoa_id_mut(w.delegate())
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

pub unsafe fn cast_cocoa_id_mut<'a, T>(id: cocoa_id) -> Option<&'a mut T> where T: Sized {
	if id.is_null() {
        return None;
    }
    let mut ivar_count = 0;
    let ivars = class_copyIvarList(msg_send![id, class], &mut ivar_count);
    let ivar: &Ivar = mem::transmute(*ivars);
    let id_: &Object = mem::transmute(id);
    let saved: *mut c_void = *id_.get_ivar(ivar.name());
    Some(mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void))
}
pub unsafe fn cast_cocoa_id<'a>(id: cocoa_id) -> Option<&'a types::UiMemberBase> {
	if id.is_null() {
        return None;
    }
    let mut ivar_count = 0;
    let ivars = class_copyIvarList(msg_send![id, class], &mut ivar_count);
    let ivar: &Ivar = mem::transmute(*ivars);
    let id_: &Object = mem::transmute(id);
    let saved: *mut c_void = *id_.get_ivar(ivar.name());
    Some(mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void))
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

    msg_send![layout_manager, addTextContainer: text_container];
    msg_send![text_container, release];
    msg_send![text_storage, addLayoutManager: layout_manager];
    msg_send![layout_manager, release];

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
			
			let parent_hwnd = this.parent_cocoa_id();	
			if let Some(parent_hwnd) = parent_hwnd {
				let mparent = common::cast_cocoa_id_mut::<plygui_api::development::UiMemberCommon>(parent_hwnd).unwrap();
				let (pw, ph) = mparent.size();
				let this: &mut $typ = mem::transmute(this);
				//let (_,_,changed) = 
				this.measure(pw, ph);
				this.draw(None);		
						
				if mparent.is_control().is_some() {
					let wparent = common::cast_cocoa_id_mut::<common::CocoaControlBase>(parent_hwnd);
					//if changed {
						//wparent.invalidate();
					//} 
				}
				/*if parent_hwnd != 0 as winapi::HWND {
		    		user32::InvalidateRect(parent_hwnd, ptr::null_mut(), winapi::TRUE);
		    	}*/
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