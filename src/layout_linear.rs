use super::*;

use plygui_api::{layout, ids, types, development, controls};
use plygui_api::development::{Drawable, HasInner};

use self::cocoa::foundation::{NSRect, NSSize, NSPoint};
use self::cocoa::base::id as cocoa_id;

use std::mem;

lazy_static! {
	static ref WINDOW_CLASS: common::RefClass = unsafe { common::register_window_class("PlyguiLinearLayout", "NSView", |_|{}) };
}
const DEFAULT_PADDING: i32 = 6;

pub type LinearLayout = development::Member<development::Control<development::MultiContainer<CocoaLinearLayout>>>;

#[repr(C)]
pub struct CocoaLinearLayout {
    base: common::CocoaControlBase<LinearLayout>,
    orientation: layout::Orientation,
    gravity_horizontal: layout::Gravity,
    gravity_vertical: layout::Gravity,
    children: Vec<Box<controls::Control>>,
}

impl development::LinearLayoutInner for CocoaLinearLayout {
	fn with_orientation(orientation: layout::Orientation) -> Box<controls::LinearLayout> {
		use plygui_api::controls::HasLayout;
		
		let mut ll = Box::new(development::Member::with_inner(development::Control::with_inner(development::MultiContainer::with_inner(CocoaLinearLayout {
                     base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                     orientation: orientation,
                     gravity_horizontal: Default::default(),
				    gravity_vertical: Default::default(),
				    children: Vec::new(),
                 }, ()), ()), development::MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)));
        let selfptr = ll.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe { (&mut *ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr); }
		ll.set_layout_padding(layout::BoundarySize::AllTheSame(DEFAULT_PADDING).into());
        ll
	}
}
impl development::MultiContainerInner for CocoaLinearLayout {
	fn len(&self) -> usize {
		self.children.len()
	}
    fn set_child_to(&mut self, base: &mut development::MemberBase, index: usize, new: Box<controls::Control>) -> Option<Box<controls::Control>> {
    	let mut old = self.remove_child_from(base, index);

        unsafe {
	        if let Some(ref mut old) = old {
		        let () = msg_send![old.native_id() as cocoa_id, removeFromSuperview];
	        }
            let () = msg_send![self.base.control, addSubview: new.native_id() as cocoa_id];
        }
        self.children.insert(index, new);

        old
    }
    fn remove_child_from(&mut self, _: &mut development::MemberBase, index: usize) -> Option<Box<controls::Control>> {
    	if index >= self.children.len() {
            return None;
        }
        let mut child = self.children.remove(index);
        child.on_removed_from_container(unsafe { common::member_from_cocoa_id::<LinearLayout>(self.base.control).unwrap() });
        
        Some(child)
    }
    fn child_at(&self, index: usize) -> Option<&controls::Control> {
    	self.children.get(index).map(|c| c.as_ref())
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut controls::Control> {
    	if let Some(c) = self.children.get_mut(index) {
        	Some(c.as_mut())
        } else {
        	None
        }
    }
}

impl development::ContainerInner for CocoaLinearLayout {
	fn find_control_by_id_mut(&mut self, id: ids::Id) -> Option<&mut controls::Control> {
		for child in self.children.as_mut_slice() {
            if child.id() == id {
                return Some(child.as_mut());
            } else if let Some(c) = child.is_container_mut() {
                let ret = c.find_control_by_id_mut(id);
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
	}
    fn find_control_by_id(&self, id: ids::Id) -> Option<&controls::Control> {
    	for child in self.children.as_slice() {
            if child.id() == id {
                return Some(child.as_ref());
            } else if let Some(c) = child.is_container() {
                let ret = c.find_control_by_id(id);
                if ret.is_none() {
                    continue;
                }
                return ret;
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

impl development::HasOrientationInner for CocoaLinearLayout {
	fn layout_orientation(&self) -> layout::Orientation {
		self.orientation
	}
    fn set_layout_orientation(&mut self, _: &mut development::MemberBase, orientation: layout::Orientation) {
    	if orientation != self.orientation {
    		self.orientation = orientation;
    		self.base.invalidate();
    	}
    }
}

impl development::ControlInner for CocoaLinearLayout {
	fn on_added_to_container(&mut self, base: &mut development::MemberControlBase, parent: &controls::Container, x: i32, y: i32) {
		let (pw, ph) = parent.draw_area_size();
        self.measure(base, pw, ph);
        //self.base.dirty = false;
        let orientation = self.orientation;
        let (lp,tp,_,_) = base.control.layout.padding.into();
    	let (lm,tm,_,_) = base.control.layout.margin.into();
        let mut x = x + lp + lm;
        let mut y = y + tp + tm;
        for ref mut child in self.children.as_mut_slice() {
            let self2 = unsafe { common::member_from_cocoa_id_mut::<LinearLayout>(self.base.control).unwrap() };
            unsafe { let () = msg_send![self2.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview: child.native_id() as cocoa_id]; }
            child.on_added_to_container(self2, x, y);
            let (xx, yy) = child.size();
            match orientation {
                layout::Orientation::Horizontal => x += xx as i32,
                layout::Orientation::Vertical => y += yy as i32,
            }
        }
	}
    fn on_removed_from_container(&mut self, _: &mut development::MemberControlBase, _: &controls::Container) {
    	let ll2: &LinearLayout = unsafe { common::member_from_cocoa_id(self.base.control).unwrap() };
        for ref mut child in self.children.as_mut_slice() {
            child.on_removed_from_container(ll2);
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
    	use plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT;
    	
    	fill_from_markup_base!(self, base, markup, registry, LinearLayout, [MEMBER_TYPE_LINEAR_LAYOUT]);
		fill_from_markup_children!(self, &mut base.member, markup, registry);	
    }
}

impl development::HasLayoutInner for CocoaLinearLayout {
	fn on_layout_changed(&mut self, _: &mut development::MemberBase) {
		self.base.invalidate();
	}
}

impl development::MemberInner for CocoaLinearLayout {
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

impl development::Drawable for CocoaLinearLayout {
	fn draw(&mut self, base: &mut development::MemberControlBase, coords: Option<(i32, i32)>) {
		use plygui_api::development::ControlInner;
		
		if coords.is_some() {
    		self.base.coords = coords;
    	}
    	if let Some((x, y)) = self.base.coords {
    		let (_, ph) = self.parent().unwrap().is_container().unwrap().size();
	        let (lp, tp, _, _) = base.control.layout.padding.into();
	        let (lm, tm, rm, bm) = base.control.layout.margin.into();
	        unsafe {
	        	let mut frame: NSRect = msg_send![self.base.control, frame];
	            frame.size = NSSize::new((self.base.measured_size.0 as i32 - lm - rm) as f64,
	                                     (self.base.measured_size.1 as i32 - bm - tm) as f64);
	            frame.origin = NSPoint::new((x + lm) as f64, (ph as i32 - y - self.base.measured_size.1 as i32 - tm) as f64);
	            let () = msg_send![self.base.control, setFrame: frame];
	        }
	        
	        let mut x = lp;
	        let mut y = tp;
	        
	        for mut child in self.children.as_mut_slice() {
	            let child_size = child.size();
	            child.draw(Some((x, y)));      
	            match self.orientation {
	                layout::Orientation::Horizontal => {
	                    x += child_size.0 as i32
	                }
	                layout::Orientation::Vertical => {
	                	y += child_size.1 as i32
	                }
	            }  
	        }    	
	        if let Some(ref mut cb) = base.member.handler_resize {
	            unsafe {
	                let mut ll2 = common::member_from_cocoa_id_mut::<LinearLayout>(self.base.control).unwrap();
	                (cb.as_mut())(ll2, self.base.measured_size.0, self.base.measured_size.1);
	            }
	        }
	    }
	}
    fn measure(&mut self, base: &mut development::MemberControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	use std::cmp::max;
    	
    	let orientation = self.orientation;
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
		                for child in self.children.as_mut_slice() {
		                    let (cw, _, _) = child.measure(
		                    	max(0, parent_width as i32 - hp) as u16, 
		                    	max(0, parent_height as i32 - vp) as u16
		                    );
		                    match orientation {
		                    	layout::Orientation::Horizontal => {
			                    	w += cw;
			                    },
		                    	layout::Orientation::Vertical => {
			                    	w = max(w, cw);
			                    },
		                    }
		                }
	        			measured = true;
	        			max(0, w as i32 + hp) as u16
        			}
        		};
        		let h = match base.control.layout.height {
        			layout::Size::Exact(h) => h,
        			layout::Size::MatchParent => parent_height,
        			layout::Size::WrapContent => {
	        			let mut h = 0;
		                for child in self.children.as_mut_slice() {
		                    let ch = if measured {
		                    	child.size().1
		                    } else {
		                    	let (_, ch, _) = child.measure(
			                    	max(0, parent_width as i32 - hp) as u16, 
			                    	max(0, parent_height as i32 - vp) as u16
			                    );
		                    	ch
		                    };
		                    match orientation {
		                    	layout::Orientation::Horizontal => {
			                    	h = max(h, ch);
			                    },
		                    	layout::Orientation::Vertical => {
			                    	h += ch;
			                    },
		                    }
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
    LinearLayout::with_orientation(layout::Orientation::Vertical).into_control()
}

impl_all_defaults!(LinearLayout);