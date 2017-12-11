use super::*;

use std::{ptr, mem, str};
use std::os::raw::c_void;
use self::cocoa::base::{class, id as cocoa_id};
use self::cocoa::foundation::{NSString, NSRect, NSRange};
use self::cocoa::appkit::{NSWindow, NSView};
use objc::runtime::{Class, Object, Ivar, YES, NO, class_copyIvarList};

use {development, ids, layout, Id, UiContainer, UiMember, UiControl, UiRoleMut, UiRole, Visibility};

pub struct RefClass(pub *const Class);
unsafe impl Sync for RefClass {}

pub const IVAR: &str = "plyguiIvar";

pub unsafe trait CocoaControl: UiMember {
    unsafe fn on_added_to_container(&mut self, &CocoaContainer, x: u16, y: u16);
    unsafe fn on_removed_from_container(&mut self, &CocoaContainer);
    fn as_base(&self) -> &CocoaControlBase;
    fn as_base_mut(&mut self) -> &mut CocoaControlBase;
}

#[repr(C)]
pub struct CocoaControlBase {
    id: Id,
    layout: development::layout::LayoutBase,
    visibility: Visibility,
    
    pub control: cocoa_id,
    pub coords: Option<(i32, i32)>,
    pub measured_size: (u16, u16),
    pub h_resize: Option<Box<FnMut(&mut UiMember, u16, u16)>>,
}

impl Default for CocoaControlBase {
    fn default() -> CocoaControlBase {
        CocoaControlBase {
        	id: ids::next(),
            control: ptr::null_mut(),
            h_resize: None,
            visibility: Visibility::Visible,
            layout: development::layout::LayoutBase {
	            width: layout::Size::MatchParent,
				height: layout::Size::WrapContent,
				gravity: layout::gravity::CENTER_HORIZONTAL | layout::gravity::TOP,
				orientation: layout::Orientation::Vertical,
				alignment: layout::Alignment::None,
            },
            coords: None,
            measured_size: (0, 0),
        }
    }
}
impl CocoaControlBase {
    pub unsafe fn on_removed_from_container(&mut self) {
        self.control.removeFromSuperview();
        msg_send![self.control, dealloc];
        self.control = ptr::null_mut();
    }
    fn invalidate(&mut self) {
		/*unsafe {
			let self_control = self.control;
			if let Some(parent) = self.parent_mut() {
				if let Some(real_self) = cast_cocoa_id_to_uimember(self_control) {
					if let Some(control_self) = real_self.is_control_mut() {
						let (pw, ph) = parent.size();
						let wparent = cast_cocoa_id_to_cocoa(parent.native_id());
						let (_,_,changed) = control_self.measure(pw, ph);
						control_self.draw(None);
							
						if changed {
							if let Some(wparent) = wparent {
								wparent.as_base_mut().invalidate();
							} 
						} 
					}
				}
	        }
		}*/
	}
    pub fn layout_width(&self) -> layout::Size {
    	self.layout.width
    }
	pub fn layout_height(&self) -> layout::Size {
		self.layout.height
	}
	pub fn layout_gravity(&self) -> layout::Gravity {
		self.layout.gravity
	}
	pub fn layout_orientation(&self) -> layout::Orientation {
		self.layout.orientation
	}
	pub fn layout_alignment(&self) -> layout::Alignment {
		self.layout.alignment
	}
	
	pub fn set_layout_width(&mut self, width: layout::Size) {
		self.layout.width = width;
		self.invalidate();
	}
	pub fn set_layout_height(&mut self, height: layout::Size) {
		self.layout.height = height;
		self.invalidate();
	}
	pub fn set_layout_gravity(&mut self, gravity: layout::Gravity) {
		self.layout.gravity = gravity;
		self.invalidate();
	}
	pub fn set_layout_orientation(&mut self, orientation: layout::Orientation) {
		self.layout.orientation = orientation;
		self.invalidate();
	}
	pub fn set_layout_alignment(&mut self, alignment: layout::Alignment) {
		self.layout.alignment = alignment;
		self.invalidate();
	}    
    pub fn set_visibility(&mut self, visibility: Visibility) {
        if self.visibility != visibility {
            self.visibility = visibility;
            unsafe {
                match self.visibility {
                    Visibility::Invisible => {
                        msg_send![self.control, setHidden: YES];
                    }
                    _ => {
                        msg_send![self.control, setHidden: NO];
                    }
                }
            }
            self.invalidate();
        }
    }
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
    pub fn id(&self) -> Id {
        self.id
    }
    pub fn parent(&self) -> Option<&UiContainer> {
        unsafe {
            let id_: cocoa_id = msg_send![self.control, superview];
            if id_.is_null() {
                return None;
            }
            let mut ivar_count = 0;
            let ivars = class_copyIvarList(msg_send![id_, class], &mut ivar_count);
            /*let ivars = from_raw_parts(ivars, ivar_count as usize);
			for ivar in ivars {
				let ivar: &Ivar = mem::transmute(*ivar);
				println!("ivar {}", ivar.name());
			}
			if ivar_count < 1 {
				return None;
			}*/
            let ivar: &Ivar = mem::transmute(*ivars);
            let id_: &Object = mem::transmute(id_);
            let saved: *mut c_void = *id_.get_ivar(ivar.name());
            match ivar.name() {
                super::layout_linear::IVAR => {
                    let ll: &LinearLayout = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
                    Some(ll)
                }
                super::window::IVAR => {
                    let w: &Window = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
                    Some(w)
                }
                _ => None,
            }
        }
    }
    pub fn parent_mut(&mut self) -> Option<&mut UiContainer> {
        unsafe {
            let id_: cocoa_id = msg_send![self.control, superview];
            cast_cocoa_id_to_uicontainer(id_)
        }
    }
    pub fn root(&self) -> Option<&UiContainer> {
        unsafe {
            let w: cocoa_id = msg_send![self.control, window];
            if w.is_null() {
                return None;
            }
            let dlg = w.delegate();
            let mut ivar_count = 0;
            let ivars = class_copyIvarList(msg_send![dlg, class], &mut ivar_count);
            let ivar: &Ivar = mem::transmute(*ivars);
            let id_: &Object = mem::transmute(dlg);
            let saved: *mut c_void = *id_.get_ivar(ivar.name());
            match ivar.name() {
                super::layout_linear::IVAR => {
                    let ll: &LinearLayout = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
                    Some(ll)
                }
                super::window::IVAR => {
                    let w: &Window = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
                    Some(w)
                }
                _ => None,
            }
        }
    }
    pub fn root_mut(&mut self) -> Option<&mut UiContainer> {
        unsafe {
            let w: cocoa_id = msg_send![self.control, window];
            cast_cocoa_id_to_uicontainer(w)
        }
    }
}

pub unsafe trait CocoaContainer: UiContainer + UiMember {
    unsafe fn cocoa_id(&self) -> cocoa_id;
}

pub unsafe fn cast_uicontrol_to_cocoa_mut(input: &mut Box<UiControl>) -> &mut CocoaControl {
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
}

pub unsafe fn cast_cocoa_id_to_uimember<'a>(id: cocoa_id) -> Option<&'a mut UiMember> {
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
        super::button::IVAR => {
            let w: &mut Button = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
            Some(w)
        },
        super::window::IVAR => {
            let w: &mut Window = mem::transmute(saved as *mut _ as *mut ::std::os::raw::c_void);
            Some(w)
        }
        _ => None,
    }
}

pub unsafe fn cast_cocoa_id_to_uicontainer<'a>(id: cocoa_id) -> Option<&'a mut UiContainer> {
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
}

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
