use super::*;
use super::common::*;

use plygui_api::{layout, ids, types, development, callbacks};
use plygui_api::traits::{UiControl, UiHasLayout, UiHasOrientation, UiMultiContainer, UiLinearLayout, UiMember, UiContainer};
use plygui_api::members::MEMBER_ID_LAYOUT_LINEAR;

use self::cocoa::foundation::{NSRect, NSSize, NSPoint};
use self::cocoa::base::id as cocoa_id;
use objc::runtime::{Class, Object};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;

lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}
const DEFAULT_PADDING: i32 = 6;

#[repr(C)]
pub struct LinearLayout {
    base: CocoaControlBase,
    orientation: layout::Orientation,
    children: Vec<Box<UiControl>>,
}

impl LinearLayout {
    pub fn new(orientation: layout::Orientation) -> Box<LinearLayout> {
        let mut ll = Box::new(LinearLayout {
                     base: common::CocoaControlBase::with_params(
			                     invalidate_impl,
                             	 development::UiMemberFunctions {
		                             fn_member_id: member_id,
								     fn_is_control: is_control,
								     fn_is_control_mut: is_control_mut,
								     fn_size: size,
	                             },
                             ),
                     orientation: orientation,
                     children: Vec::new(),
                 });
        ll.set_layout_padding(layout::BoundarySize::AllTheSame(DEFAULT_PADDING).into());
        ll
    }
}
impl UiMember for LinearLayout {
	fn is_control(&self) -> Option<&UiControl> {
    	Some(self)
    }
    fn is_control_mut(&mut self) -> Option<&mut UiControl> {
    	Some(self)
    }
    fn set_visibility(&mut self, visibility: types::Visibility) {
        self.base.set_visibility(visibility);
    }
    fn visibility(&self) -> types::Visibility {
        self.base.visibility()
    }
    fn size(&self) -> (u16, u16) {
        self.base.measured_size
    }
	fn on_resize(&mut self, handler: Option<callbacks::Resize>) {
        self.base.h_resize = handler;
    }

    unsafe fn native_id(&self) -> usize {
        self.base.control as usize
    }
    fn as_base(&self) -> &types::UiMemberBase {
    	self.base.control_base.member_base.as_ref()
    }
    fn as_base_mut(&mut self) -> &mut types::UiMemberBase {
    	self.base.control_base.member_base.as_mut()
    }
}

impl UiHasLayout for LinearLayout {
	fn layout_width(&self) -> layout::Size {
	    	self.base.control_base.layout.width
    }
	fn layout_height(&self) -> layout::Size {
		self.base.control_base.layout.height
	}
	fn layout_gravity(&self) -> layout::Gravity {
		self.base.control_base.layout.gravity
	}
	fn layout_alignment(&self) -> layout::Alignment {
		self.base.control_base.layout.alignment
	}
	fn layout_padding(&self) -> layout::BoundarySize {
		self.base.control_base.layout.padding
	}
    fn layout_margin(&self) -> layout::BoundarySize {
	    	self.base.control_base.layout.margin
    }

    fn set_layout_padding(&mut self, padding: layout::BoundarySizeArgs) {
	    	self.base.control_base.layout.padding = padding.into();
		self.base.invalidate();
    }
    fn set_layout_margin(&mut self, margin: layout::BoundarySizeArgs) {
	    	self.base.control_base.layout.margin = margin.into();
		self.base.invalidate();
    }
	fn set_layout_width(&mut self, width: layout::Size) {
		self.base.control_base.layout.width = width;
		self.base.invalidate();
	}
	fn set_layout_height(&mut self, height: layout::Size) {
		self.base.control_base.layout.height = height;
		self.base.invalidate();
	}
	fn set_layout_gravity(&mut self, gravity: layout::Gravity) {
		self.base.control_base.layout.gravity = gravity;
		self.base.invalidate();
	}
	fn set_layout_alignment(&mut self, alignment: layout::Alignment) {
		self.base.control_base.layout.alignment = alignment;
		self.base.invalidate();
	} 
	fn as_member(&self) -> &UiMember {
		self
	}
	fn as_member_mut(&mut self) -> &mut UiMember {
		self
	}  
}

impl UiControl for LinearLayout {
    fn on_added_to_container(&mut self, parent: &UiContainer, x: i32, y: i32) {
    	use plygui_api::development::UiDrawable;
    	
        let (pw, ph) = parent.draw_area_size();
        let (w, h, _) = self.measure(pw, ph);
		let (lp, tp, _, _) = self.base.control_base.layout.padding.into();
        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();
        
        let rect = NSRect::new(NSPoint::new((x + lm) as f64, (ph as i32 - y - tm - h as i32) as f64),
                               NSSize::new((w as i32 - rm - lm) as f64, (h as i32 - tm - bm) as f64));

        unsafe {
        	let base: cocoa_id = msg_send![WINDOW_CLASS.0, alloc];
	        let base: cocoa_id = msg_send![base, initWithFrame: rect];
			self.base.coords = Some((x as i32, y as i32));
	        self.base.control = msg_send![base, autorelease];
	        (&mut *self.base.control).set_ivar(IVAR, self as *mut _ as *mut ::std::os::raw::c_void);
	
	        let mut x = lp;
	        let mut y = tp;
	        let ll2: &LinearLayout = mem::transmute(self as *mut _ as *mut ::std::os::raw::c_void);
	        for ref mut child in self.children.as_mut_slice() {
	            child.on_added_to_container(ll2, x, y);
	            let (xx, yy) = child.size();
	            match self.orientation {
	                layout::Orientation::Horizontal => {
	                    x += xx as i32;
	                }
	                layout::Orientation::Vertical => {
	                    y += yy as i32;
	                }
	            }
	            let () = msg_send![ll2.base.control, addSubview: child.native_id() as cocoa_id];
	        }
        }
    }
    fn on_removed_from_container(&mut self, _: &UiContainer) {
        let ll2: &LinearLayout = unsafe { mem::transmute(self as *mut _ as *mut ::std::os::raw::c_void) };
        for ref mut child in self.children.as_mut_slice() {
            child.on_removed_from_container(ll2);
        }
        unsafe { self.base.on_removed_from_container(); }
    }
    fn is_container_mut(&mut self) -> Option<&mut UiContainer> {
        Some(self)
    }
    fn is_container(&self) -> Option<&UiContainer> {
        Some(self)
    }
    fn parent(&self) -> Option<&types::UiMemberBase> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&types::UiMemberBase> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut types::UiMemberBase> {
        self.base.root_mut()
    }
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
    	use plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT;
    	
    	fill_from_markup_base!(self, markup, registry, LinearLayout, [MEMBER_ID_LAYOUT_LINEAR, MEMBER_TYPE_LINEAR_LAYOUT]);
		fill_from_markup_children!(self, markup, registry);		
    }
    fn as_has_layout(&self) -> &UiHasLayout {
    	self
    }
	fn as_has_layout_mut(&mut self) -> &mut UiHasLayout {
		self
	}
}

impl UiMultiContainer for LinearLayout {
    fn push_child(&mut self, child: Box<UiControl>) {
        let len = self.children.len();
        self.set_child_to(len, child);
    }
    fn pop_child(&mut self) -> Option<Box<UiControl>> {
        let len = self.children.len();
        self.remove_child_from(len - 1)
    }
    fn len(&self) -> usize {
        self.children.len()
    }
    fn set_child_to(&mut self, index: usize, mut new: Box<UiControl>) -> Option<Box<UiControl>> {
        let old = self.remove_child_from(index);

        if !self.base.control.is_null() {
            let (lp, tp, _, _) = self.base.control_base.layout.padding.into();
	        let (lm, tm, _, _) = self.base.control_base.layout.margin.into();
	        let (x, y) = {
                let mut x = lm + lp;
                let mut y = tm + tp;
                for ref child in self.children.as_slice() {
                    let (xx, yy) = child.size();
                    match self.orientation {
                        layout::Orientation::Horizontal => x += xx as i32,
                        layout::Orientation::Vertical => y += yy as i32,
                    }
                }
                (x, y)
            };
            unsafe {
                //let mut wc = common::cast_uicontrol_to_cocoa_mut(&mut new);
                let (_,yy) = new.size();
                match self.orientation {
                    layout::Orientation::Horizontal => {
                        new.on_added_to_container(self, x as i32, y as i32); //TODO padding
                    }
                    layout::Orientation::Vertical => {
                        let my_h = self.size().1;
                        new.on_added_to_container(self, x as i32, my_h as i32 - y as i32 - yy as i32); //TODO padding
                    }
                }
                let () = msg_send![self.base.control, addSubview: new.native_id() as cocoa_id];
            }
        }
        self.children.insert(index, new);

        old
    }
    fn remove_child_from(&mut self, index: usize) -> Option<Box<UiControl>> {
        if index >= self.children.len() {
            return None;
        }
        let mut child = self.children.remove(index);
        child.on_removed_from_container(self);
        
        Some(child)
    }
    fn child_at(&self, index: usize) -> Option<&Box<UiControl>> {
        self.children.get(index)
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut Box<UiControl>> {
        self.children.get_mut(index)
    }
    fn as_container(&self) -> &UiContainer {
    	self
    }
	fn as_container_mut(&mut self) -> &mut UiContainer {
		self
	}
}

impl UiContainer for LinearLayout {
    fn is_multi_mut(&mut self) -> Option<&mut UiMultiContainer> {
        Some(self)
    }
    fn is_multi(&self) -> Option<&UiMultiContainer> {
        Some(self)
    }
    fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut UiControl> {
        if self.as_base().id() == id_ {
            return Some(self);
        }
        for child in self.children.as_mut_slice() {
            if child.as_base().id() == id_ {
                return Some(child.as_mut());
            } else if let Some(c) = child.is_container_mut() {
                let ret = c.find_control_by_id_mut(id_);
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&UiControl> {
        if self.as_base().id() == id_ {
            return Some(self);
        }
        for child in self.children.as_slice() {
            if child.as_base().id() == id_ {
                return Some(child.as_ref());
            } else if let Some(c) = child.is_container() {
                let ret = c.find_control_by_id(id_);
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
    }
    fn as_member(&self) -> &UiMember {
    	self
    }
	fn as_member_mut(&mut self) -> &mut UiMember {
		self
	}
}

impl UiHasOrientation for LinearLayout {
	fn layout_orientation(&self) -> layout::Orientation {
        self.orientation
    }
    fn set_layout_orientation(&mut self, orientation: layout::Orientation) {
        self.orientation = orientation;
    }
}

impl UiLinearLayout for LinearLayout {
    fn as_control(&self) -> &UiControl {
	    	self
    }
	fn as_control_mut(&mut self) -> &mut UiControl {
		self
	}
	fn as_multi_container(&self) -> &UiMultiContainer {
		self
	}
	fn as_multi_container_mut(&mut self) -> &mut UiMultiContainer {
		self
	}
	fn as_has_orientation(&self) -> &UiHasOrientation {
		self
	}
	fn as_has_orientation_mut(&mut self) -> &mut UiHasOrientation {
		self
	}
}

impl development::UiDrawable for LinearLayout {
	fn draw(&mut self, coords: Option<(i32, i32)>) {
    	if coords.is_some() {
    		self.base.coords = coords;
    	}
    	if let Some((x, y)) = self.base.coords {
    		let (_, ph) = self.parent().unwrap().as_ref().size();
	        let (lp, tp, _, _) = self.base.control_base.layout.padding.into();
	        let (lm, tm, rm, bm) = self.base.control_base.layout.margin.into();
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
	        if let Some(ref mut cb) = self.base.h_resize {
	            unsafe {
	                let object: &Object = mem::transmute(self.base.control);
	                let saved: *mut c_void = *object.get_ivar(IVAR);
	                let mut ll2: &mut LinearLayout = mem::transmute(saved);
	                (cb.as_mut())(ll2, self.base.measured_size.0, self.base.measured_size.1);
	            }
	        }
	    }
    }
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	use std::cmp::max;
    	
    	let orientation = self.layout_orientation();
    	let old_size = self.base.measured_size;
    	let (lp,tp,rp,bp) = self.base.control_base.layout.padding.into();
    	let (lm,tm,rm,bm) = self.base.control_base.layout.margin.into();
    	let hp = lm + rm + lp + rp;
    	let vp = tm + bm + tp + bp;
    	self.base.measured_size = match self.visibility() {
        	types::Visibility::Gone => (0,0),
        	_ => {
        		let mut measured = false;
        		let w = match self.layout_width() {
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
        		let h = match self.layout_height() {
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
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<UiControl> {
	LinearLayout::new(layout::Orientation::Vertical)
}

unsafe fn register_window_class() -> RefClass {
    let superclass = Class::get("NSView").unwrap();
    let mut decl = ClassDecl::new(MEMBER_ID_LAYOUT_LINEAR, superclass).unwrap();

    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}
impl_invalidate!(LinearLayout);
impl_is_control!(LinearLayout);
impl_size!(LinearLayout);
impl_member_id!(MEMBER_ID_LAYOUT_LINEAR);