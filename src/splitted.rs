use super::*;
use super::common::*;

lazy_static! {
	static ref WINDOW_CLASS: common::RefClass = unsafe { common::register_window_class("PlyguiSplitted", "NSSplitView", |_|{}) };
}

pub type Splitted = Member<Control<MultiContainer<CocoaSplitted>>>;

#[repr(C)]
pub struct CocoaSplitted {
    base: common::CocoaControlBase<Splitted>,
    orientation: layout::Orientation,
    splitter: f32,
    first: Box<controls::Control>,
    second: Box<controls::Control>,
}

impl SplittedInner for CocoaSplitted {
	fn with_content(first: Box<dyn controls::Control>, second: Box<dyn controls::Control>, orientation: layout::Orientation) -> Box<Splitted> {
		let mut ll = Box::new(Member::with_inner(Control::with_inner(MultiContainer::with_inner(CocoaSplitted {
                     base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                     orientation: orientation,
                     splitter: defaults::SPLITTED_POSITION,
				    first: first,
				    second: second,
                 }, ()), ()), MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut)));
        let selfptr = ll.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        let vertical = match orientation {
	        layout::Orientation::Horizontal => YES,
	        layout::Orientation::Vertical => NO,
        };
        unsafe { 
        	(&mut *ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr); 
        	let first = ll.as_inner_mut().as_inner_mut().as_inner_mut().first.native_id() as cocoa_id;
        	let second = ll.as_inner_mut().as_inner_mut().as_inner_mut().second.native_id() as cocoa_id;
	        let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview:first];
	        let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview:second];
	        let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, setVertical:vertical];
	        let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, adjustSubviews];
        }
		ll
	}
    fn set_splitter(&mut self, member: &mut MemberBase, control: &mut ControlBase, pos: f32) {
    	self.splitter = pos;
	    self.base.invalidate();
	}
	fn splitter(&self) -> f32 {
		self.splitter
	}
	fn first(&self) -> &controls::Control { self.first.as_ref() }
	fn second(&self) -> &controls::Control { self.second.as_ref() }
	fn first_mut(&mut self) -> &mut controls::Control { self.first.as_mut() }
	fn second_mut(&mut self) -> &mut controls::Control { self.second.as_mut() }
}
impl MultiContainerInner for CocoaSplitted {
	fn len(&self) -> usize {
		2
	}
    fn set_child_to(&mut self, base: &mut MemberBase, index: usize, mut child: Box<controls::Control>) -> Option<Box<controls::Control>> {
    	match index {
	    	0 => unsafe {
	    		let self2 = utils::base_to_impl_mut::<Splitted>(base);		
		    	let sizes = self.first.size();
		    	let () = msg_send![self.first.native_id() as cocoa_id, removeFromSuperview];		    
			    self.first.on_removed_from_container(self2);
			    let () = msg_send![child.native_id() as cocoa_id, addSubview: child.native_id() as cocoa_id];
			    child.on_added_to_container(self2, 0, 0, sizes.0, sizes.1);
	    		mem::swap(&mut self.first, &mut child);
	    	},
	    	1 => unsafe {
	    		let self2 = utils::base_to_impl_mut::<Splitted>(base);
			    let sizes = self.second.size();		
			    let () = msg_send![self.second.native_id() as cocoa_id, removeFromSuperview];
	    		self.second.on_removed_from_container(self2);
	    		let () = msg_send![child.native_id() as cocoa_id, addSubview: child.native_id() as cocoa_id];
	    		child.on_added_to_container(self2, 0, 0, sizes.0, sizes.1);
	    		mem::swap(&mut self.second, &mut child);
	    	},
	    	_ => return None,
    	}
    	
    	Some(child)
    }
    fn remove_child_from(&mut self, _: &mut MemberBase, index: usize) -> Option<Box<controls::Control>> {
    	None
    }
    fn child_at(&self, index: usize) -> Option<&controls::Control> {
    	match index {
    		0 => Some(self.first()),
    		1 => Some(self.second()),
    		_ => None
    	}
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut controls::Control> {
    	match index {
    		0 => Some(self.first_mut()),
    		1 => Some(self.second_mut()),
    		_ => None
    	}
    }
}

impl ContainerInner for CocoaSplitted {
	fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut controls::Control> {
		if self.first().as_member().id() == id_ {
			return Some(self.first_mut());
		}
		if self.second().as_member().id() == id_ {
			return Some(self.second_mut());
		}
		
		let self2: &mut Self = unsafe { mem::transmute(self as *mut Self) }; // bck is stupid
		if let Some(c) = self.first_mut().is_container_mut() {
            let ret = c.find_control_by_id_mut(id_);
            if ret.is_some() {
                return ret;
            }
        }
		if let Some(c) = self2.second_mut().is_container_mut() {
            let ret = c.find_control_by_id_mut(id_);
            if ret.is_some() {
                return ret;
            }
        }
		
        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&controls::Control> {
    	if self.first().as_member().id() == id_ {
			return Some(self.first());
		}
		if self.second().as_member().id() == id_ {
			return Some(self.second());
		}
		
		if let Some(c) = self.first().is_container() {
            let ret = c.find_control_by_id(id_);
            if ret.is_some() {
                return ret;
            }
        }
		if let Some(c) = self.second().is_container() {
            let ret = c.find_control_by_id(id_);
            if ret.is_some() {
                return ret;
            }
        }
		
        None
    }
}

impl HasOrientationInner for CocoaSplitted {
	fn layout_orientation(&self) -> layout::Orientation {
		self.orientation
	}
    fn set_layout_orientation(&mut self, _: &mut MemberBase, orientation: layout::Orientation) {
    	if orientation != self.orientation {
    		self.orientation = orientation;
    		self.base.invalidate();
    	}
    }
}

impl ControlInner for CocoaSplitted {
	fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
		self.measure(member, control, pw, ph);
        let self2: &mut Splitted = unsafe { utils::base_to_impl_mut(member) };
        let (first_size, second_size) = (pw, ph);
        match self.orientation {
            layout::Orientation::Horizontal => {
                let h = utils::coord_to_size(ph as i32);
                unsafe { let () = msg_send![self2.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview: self.first.native_id() as cocoa_id]; }
            	self.first.on_added_to_container(self2, 0, 0, first_size, h); 
            	unsafe { let () = msg_send![self2.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview: self.second.native_id() as cocoa_id]; }
                self.second.on_added_to_container(self2, 0 + first_size as i32, 0, second_size, h); 
            },
            layout::Orientation::Vertical => {
                let w = utils::coord_to_size(pw as i32);
                unsafe { let () = msg_send![self2.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview: self.first.native_id() as cocoa_id]; }
            	self.first.on_added_to_container(self2, 0, 0, w, first_size); 
            	unsafe { let () = msg_send![self2.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview: self.second.native_id() as cocoa_id]; }
                self.second.on_added_to_container(self2, 0, 0 + first_size as i32, w, second_size);
            },
        }
	}
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &controls::Container) {
    	let self2: &Splitted = unsafe { common::member_from_cocoa_id(self.base.control).unwrap() };
        self.first.on_removed_from_container(self2);    
        self.second.on_removed_from_container(self2); 
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
    	use plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT;
    	
    	fill_from_markup_base!(self, base, markup, registry, Splitted, [MEMBER_TYPE_LINEAR_LAYOUT]);
		fill_from_markup_children!(self, &mut base.member, markup, registry);	
    }
}

impl HasLayoutInner for CocoaSplitted {
	fn on_layout_changed(&mut self, _: &mut MemberBase) {
		self.base.invalidate();
	}
}

impl MemberInner for CocoaSplitted {
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

impl Drawable for CocoaSplitted {
	fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
		if coords.is_some() {
    		self.base.coords = coords;
    	}
    	if let Some((x, y)) = self.base.coords {
    		let (_, ph) = self.parent().unwrap().is_container().unwrap().size();
	        unsafe {
	        	let mut frame: NSRect = msg_send![self.base.control, frame];
	            frame.size = NSSize::new((self.base.measured_size.0 as i32) as f64,
	                                     (self.base.measured_size.1 as i32) as f64);
	            frame.origin = NSPoint::new(x as f64, (ph as i32 - y - self.base.measured_size.1 as i32) as f64);
	            let () = msg_send![self.base.control, setFrame: frame];
	        }
	        
	        let mut x = 0;
	        let mut y = 0;
	        
	        for child in [self.first.as_mut(), self.second.as_mut()].iter_mut() {
	            let child_size = child.size();
	            child.draw(Some((x, y)));      
	        }
	    }
	}
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	use std::cmp::max;
    	
    	let orientation = self.orientation;
    	let old_size = self.base.measured_size;
    	self.base.measured_size = match member.visibility {
        	types::Visibility::Gone => (0,0),
        	_ => {
        		let mut measured = false;
        		let w = match control.layout.width {
        			layout::Size::Exact(w) => w,
        			layout::Size::MatchParent => parent_width,
        			layout::Size::WrapContent => {
	        			let mut w = 0;
		                for child in [self.first.as_mut(), self.second.as_mut()].iter_mut() {
		                    let (cw, _, _) = child.measure(
		                    	max(0, parent_width as i32) as u16, 
		                    	max(0, parent_height as i32) as u16
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
	        			max(0, w as i32) as u16
        			}
        		};
        		let h = match control.layout.height {
        			layout::Size::Exact(h) => h,
        			layout::Size::MatchParent => parent_height,
        			layout::Size::WrapContent => {
	        			let mut h = 0;
		                for child in [self.first.as_mut(), self.second.as_mut()].iter_mut() {
		                    let ch = if measured {
		                    	child.size().1
		                    } else {
		                    	let (_, ch, _) = child.measure(
			                    	max(0, parent_width as i32) as u16, 
			                    	max(0, parent_height as i32) as u16
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
	        			max(0, h as i32) as u16
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
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
    	self.base.invalidate();
    }
}

/*#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Splitted::with_orientation(layout::Orientation::Vertical).into_control()
}*/

impl_all_defaults!(Splitted);