use super::*;

use plygui_api::{layout, ids, types, development, controls};
use plygui_api::development::{Drawable, HasInner};

use self::cocoa::foundation::{NSString, NSRect, NSSize, NSPoint};
use self::cocoa::base::id as cocoa_id;

use std::mem;
use std::os::raw::{c_char, c_void};
use std::borrow::Cow;
use std::ffi::CStr;

const INNER_PADDING: i32 = 5;

lazy_static! {
	static ref WINDOW_CLASS: common::RefClass = unsafe { common::register_window_class("PlyguiFrame", "NSBox", |_|{}) };
}

pub type Frame = development::Member<development::Control<development::SingleContainer<CocoaFrame>>>;

#[repr(C)]
pub struct CocoaFrame {
    base: common::CocoaControlBase<Frame>,
    label_padding: (i32, i32),
    gravity_horizontal: layout::Gravity,
    gravity_vertical: layout::Gravity,
    child: Option<Box<controls::Control>>,
}

impl development::FrameInner for CocoaFrame {
	fn with_label(label: &str) -> Box<controls::Frame> {
		use plygui_api::controls::HasLabel;
		
		let mut frame = Box::new(development::Member::with_inner(development::Control::with_inner(development::SingleContainer::with_inner(CocoaFrame {
                     base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                     label_padding: (0, 0),
                     gravity_horizontal: Default::default(),
				    gravity_vertical: Default::default(),
				    child: None,
                 }, ()), ())
				, development::MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)));
        let selfptr = frame.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe { (&mut *frame.as_inner_mut().as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr); }
        frame.set_label(label);
        frame
	}
}

impl CocoaFrame {
    fn measure_label(&mut self) {
        let label_size = unsafe { common::measure_nsstring(msg_send![self.base.control, title]) }; 
        self.label_padding = (label_size.0 as i32, label_size.1 as i32);
    }
}

impl development::SingleContainerInner for CocoaFrame {
	fn set_child(&mut self, _: &mut development::MemberBase, child: Option<Box<controls::Control>>) -> Option<Box<controls::Control>> {
        let mut old = self.child.take();
        self.child = child;
        if let Some(ref mut child) = self.child {
        	unsafe { 
        		let child_id = child.native_id() as cocoa_id;
	        	(&mut *child_id).set_ivar(common::IVAR_PARENT, self.base.control as *mut c_void);
	        	let () = msg_send![self.base.control, addSubview:child_id]; 
        	}
        } 
		if let Some(ref mut old) = old {
	        unsafe { let () = msg_send![old.native_id() as cocoa_id, removeFromSuperview]; }
        }
        old
    }
    fn child(&self) -> Option<&controls::Control> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut controls::Control> {
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
}

impl development::ContainerInner for CocoaFrame {
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
    fn set_gravity(&mut self, base: &mut development::MemberBase, w: layout::Gravity, h: layout::Gravity) {
    	if self.gravity_horizontal != w || self.gravity_vertical != h {
    		self.gravity_horizontal = w;
    		self.gravity_vertical = h;
    		self.invalidate(unsafe { mem::transmute(base) });
    	}
    }
}

impl development::HasLabelInner for CocoaFrame {
	fn label(&self) -> Cow<str> {
		unsafe {
			let label: cocoa_id = msg_send![self.base.control, getTitle];
			let label: *const c_void = msg_send![label, UTF8String];
	        CStr::from_ptr(label as *const c_char).to_string_lossy()
		}
    }
    fn set_label(&mut self, _: &mut development::MemberBase, label: &str) {
	    unsafe {
			let title = NSString::alloc(cocoa::base::nil).init_str(label);
        		let () = msg_send![self.base.control, setTitle:title];
            let () = msg_send![title, release];
		}
	    self.measure_label();
    }
}

impl development::ControlInner for CocoaFrame {
	fn on_added_to_container(&mut self, base: &mut development::MemberControlBase, parent: &controls::Container, _x: i32, _y: i32) {
		let (pw, ph) = parent.draw_area_size();
        self.measure(base, pw, ph);
		
		if let Some(ref mut child) = self.child {
        	let frame2 = unsafe { common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap() };
	        unsafe { let () = msg_send![frame2.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview:child.native_id() as cocoa_id]; }
	        let (lm, tm, _, _) = base.control.layout.margin.into();
	        let (lp, tp, _, _) = base.control.layout.padding.into();
            child.on_added_to_container(frame2, lp + lm, tp + tm + INNER_PADDING + self.label_padding.1 as i32);
        }
	}
    fn on_removed_from_container(&mut self, _: &mut development::MemberControlBase, _: &controls::Container) {
    	let frame2 = unsafe { common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap() };
        if let Some(ref mut child) = self.child {
            child.on_removed_from_container(frame2);
        }
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
    fn fill_from_markup(&mut self, base: &mut development::MemberControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
    	use plygui_api::markup::MEMBER_TYPE_FRAME;
    	
    	fill_from_markup_base!(self, base, markup, registry, Frame, [MEMBER_TYPE_FRAME]);
    	fill_from_markup_label!(self, &mut base.member, markup);
		fill_from_markup_child!(self, &mut base.member, markup, registry);	
    }
}

impl development::MemberInner for CocoaFrame {
	type Id = common::CocoaId;
	
    fn size(&self) -> (u16, u16) {
    	self.base.measured_size
    }
    
    fn on_set_visibility(&mut self, base: &mut development::MemberBase) {
    	self.base.on_set_visibility(base);
    }
    
    unsafe fn native_id(&self) -> Self::Id {
    	self.base.control.into()
    }
}

impl development::HasLayoutInner for CocoaFrame {
	fn on_layout_changed(&mut self, _: &mut development::MemberBase) {
		self.base.invalidate();
	}
}

impl development::Drawable for CocoaFrame {
	fn draw(&mut self, base: &mut development::MemberControlBase, coords: Option<(i32, i32)>) {
		use plygui_api::development::ControlInner;
		
    	if coords.is_some() {
    		self.base.coords = coords;
    	}
    	if let Some((x, y)) = self.base.coords {
    		let (lp, tp, _, _) = base.control.layout.padding.into();
    	    let (lm, tm, rm, bm) = base.control.layout.margin.into();
	        let (_,ph) = self.parent().unwrap().is_container().unwrap().size();
    		unsafe {
	            let mut frame: NSRect = self.base.frame();
	            frame.size = NSSize::new((self.base.measured_size.0 as i32 - lm - rm) as f64,
	                                     (self.base.measured_size.1 as i32 - tm - bm) as f64);
	            frame.origin = NSPoint::new((x + lm) as f64, (ph as i32 - (y + bm + self.base.measured_size.1 as i32)) as f64);
	            let () = msg_send![self.base.control, setFrame: frame];
	        }
    		if let Some(ref mut child) = self.child {
    	        child.draw(Some((lp, tp + INNER_PADDING + self.label_padding.1 as i32)));  
    	    }
    		if let Some(ref mut cb) = base.member.handler_resize {
	            unsafe {
	                let mut frame2 = common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap();
	                (cb.as_mut())(frame2, self.base.measured_size.0, self.base.measured_size.1);
	            }
	        }
    	}
    }
    fn measure(&mut self, base: &mut development::MemberControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	use std::cmp::max;
        	
    	let old_size = self.base.measured_size;
    	let (lp,tp,rp,bp) = base.control.layout.padding.into();
    	let (lm,tm,rm,bm) = base.control.layout.margin.into();
    	let hp = lm + rm + lp + rp;
    	let vp = tm + bm + tp + bp;
    	self.base.measured_size = match base.member.visibility {
        	types::Visibility::Gone => (0,0),
        	_ => {
        		let mut measured = false;
		        let w = match base.control.layout.width {
        			layout::Size::Exact(w) => w,
        			layout::Size::MatchParent => parent_width,
        			layout::Size::WrapContent => {
    	        			let mut w = 0;
    	        			if let Some(ref mut child) =  self.child {
    		                let (cw, _, _) = child.measure(
    		                    	max(0, parent_width as i32 - hp) as u16, 
    		                    	max(0, parent_height as i32 - vp) as u16
		                    );
		                    w += max(cw as i32, self.label_padding.0);
		                    measured = true;
		                }
    	        			max(0, w as i32 + hp) as u16
        			}
        		};
        		let h = match base.control.layout.height {
        			layout::Size::Exact(h) => h,
        			layout::Size::MatchParent => parent_height,
        			layout::Size::WrapContent => {
	        			let mut h = 0;
		                if let Some(ref mut child) =  self.child {
		                    let ch = if measured {
    		                    	child.size().1
		                    } else {
		                        let (_, ch, _) = child.measure(
    			                    	max(0, parent_width as i32 - hp) as u16, 
    			                    	max(0, parent_height as i32 - vp) as u16
    			                    );
    		                    ch	                    	
		                    };
		                    h += ch as i32 + self.label_padding.1;
		                }
	        			max(0, h as i32 + vp) as u16
        			}
        		};
        		(w, h)
        	}
        };
    	(
	        self.base.measured_size.0,
	        self.base.measured_size.1,
	        self.base.measured_size != old_size,
        )
    }
    fn invalidate(&mut self, _: &mut development::MemberControlBase) {
    	self.base.invalidate();
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Frame::with_label("").into_control()
}

impl_all_defaults!(Frame);