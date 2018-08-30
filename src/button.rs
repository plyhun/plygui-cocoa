use super::*;
use super::common::*;

use self::cocoa::appkit::NSBezelStyle;
use self::cocoa::foundation::{NSString, NSRect, NSSize, NSPoint};
use self::cocoa::base::id;
use objc::runtime::{Class, Object, Sel};

use std::mem;
use std::os::raw::{c_char, c_void};
use std::borrow::Cow;
use std::ffi::CStr;

lazy_static! {
	static ref WINDOW_CLASS: common::RefClass = unsafe { common::register_window_class("PlyguiButton", BASE_CLASS, |decl| {
			decl.add_method(sel!(mouseDown:),
                    button_left_click as extern "C" fn(&mut Object, Sel, id));
		    decl.add_method(sel!(rightMouseDown:),
		                    button_right_click as extern "C" fn(&mut Object, Sel, id));
    
		}) };
}

const DEFAULT_PADDING: i32 = 6;
const BASE_CLASS: &str = "NSButton";

pub type Button = Member<Control<CocoaButton>>;

#[repr(C)]
pub struct CocoaButton {
    base: common::CocoaControlBase<Button>,

    h_left_clicked: Option<callbacks::Click>,
    h_right_clicked: Option<callbacks::Click>,
}

impl ButtonInner for CocoaButton {
	fn with_label(label: &str) -> Box<Button> {
		use plygui_api::controls::HasLabel;
		
		let mut b = Box::new(
			Member::with_inner(
				Control::with_inner(
					CocoaButton {
	                     base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
	                     h_left_clicked: None,
	                     h_right_clicked: None,
	                 }, 
					()
				), MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)
			)
		);
        let selfptr = b.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
        	(&mut *b.as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr);
		    let () = msg_send![b.as_inner_mut().as_inner_mut().base.control, setBezelStyle: NSBezelStyle::NSSmallSquareBezelStyle]; 
        }       	    
        b.set_label(label);
        b
	}
}

impl HasLabelInner for CocoaButton {
	fn label(&self) -> Cow<str> {
		unsafe {
			let label: id = msg_send![self.base.control, title];
			let label: *const c_void = msg_send![label, UTF8String];
	        CStr::from_ptr(label as *const c_char).to_string_lossy()
		}
    }
    fn set_label(&mut self, _: &mut MemberBase, label: &str) {
	    unsafe {
			let title = NSString::alloc(cocoa::base::nil).init_str(label);
    		let () = msg_send![self.base.control, setTitle:title];
            let () = msg_send![title, release];
		}
    }
}

impl ClickableInner for CocoaButton {
	fn on_click(&mut self, cb: Option<callbacks::Click>) {
		self.h_left_clicked = cb;
	}
}

impl ControlInner for CocoaButton {
	fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
		self.measure(member, control, pw, ph);
	}
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &controls::Container) {
    	unsafe { self.base.on_removed_from_container(); }
    }
    
    fn parent(&self) -> Option<&controls::Member> {
    	self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
    	self.base.parent_mut()
    }
    fn root(&self) -> Option<&controls::Member> {
    	self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
    	self.base.root_mut()
    }
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberBase, control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
    	use plygui_api::markup::MEMBER_TYPE_BUTTON;
    	use plygui_api::ClickableInner;
    	
    	fill_from_markup_base!(self, base, markup, registry, Button, [MEMBER_TYPE_BUTTON]);
    	fill_from_markup_label!(self, &mut base.member, markup);
    	fill_from_markup_callbacks!(self, markup, registry, [on_click => plygui_api::Click]);
    }
}

impl MemberInner for CocoaButton {
	type Id = common::CocoaId;
	
    fn size(&self) -> (u16, u16) {
    	self.base.measured_size
    }
    
    fn on_set_visibility(&mut self, base: &mut MemberBase) {
    	self.base.on_set_visibility(base);
    }
    
    unsafe fn native_id(&self) -> Self::Id {
    	self.base.control.into()
    }
}

impl HasLayoutInner for CocoaButton {
	fn on_layout_changed(&mut self, _: &mut MemberBase) {
		self.base.invalidate();
	}
}

impl Drawable for CocoaButton {
	fn draw(&mut self, member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
		if coords.is_some() {
    		self.base.coords = coords;
    	}
    	if let Some((x, y)) = self.base.coords {
    		let (_,ph) = self.parent_mut().unwrap().is_container_mut().unwrap().size();
    		unsafe {
	            let mut frame: NSRect = self.base.frame();
	            frame.size = NSSize::new((self.base.measured_size.0 as i32) as f64,
	                                     (self.base.measured_size.1 as i32) as f64);
	            frame.origin = NSPoint::new(x as f64, (ph as i32 - y - self.base.measured_size.1 as i32) as f64);
	            let () = msg_send![self.base.control, setFrame: frame];
	        }
    		if let Some(ref mut cb) = member.handler_resize {
	            unsafe {
	                let object: &Object = mem::transmute(self.base.control);
	                let saved: *mut c_void = *object.get_ivar(common::IVAR);
	                let mut ll2: &mut Button = mem::transmute(saved);
	                (cb.as_mut())(ll2, self.base.measured_size.0, self.base.measured_size.1);
	            }
	        }
    	}
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	use std::cmp::max;
    	
    	let old_size = self.base.measured_size;
        self.base.measured_size = match member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size = (0, 0);
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        label_size = common::measure_nsstring(msg_send![self.base.control, title]);
                        label_size.0 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    } 
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.1 < 1 {
                            label_size = common::measure_nsstring(msg_send![self.base.control, title]);
                        }
                        label_size.1 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    } 
                };
                (max(0, w) as u16, max(0, h) as u16)
            },
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
    	self.base.invalidate();
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
	Button::with_label("").into_control()
}

extern "C" fn button_left_click(this: &mut Object, _: Sel, param: id) {
	unsafe {
        let button = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
        let () = msg_send![super(button.as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), mouseDown: param];
        if let Some(ref mut cb) = button.as_inner_mut().as_inner_mut().h_left_clicked {
            let b2 = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
            (cb.as_mut())(b2);
        }
    }
}
extern "C" fn button_right_click(this: &mut Object, _: Sel, param: id) {
    //println!("right!");
    unsafe {
        let button = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
        if let Some(ref mut cb) = button.as_inner_mut().as_inner_mut().h_right_clicked {
            let b2 = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
            (cb.as_mut())(b2);
        }
        let () = msg_send![super(button.as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), rightMouseDown: param];
    }
}

impl_all_defaults!(Button);