use super::*;
use super::common::*;

use plygui_api::{layout, ids, types, development, callbacks};
use plygui_api::traits::{UiControl, UiLayable, UiMultiContainer, UiLinearLayout, UiMember, UiContainer};
use plygui_api::members::MEMBER_ID_LAYOUT_LINEAR;

use self::cocoa::appkit::NSView;
use self::cocoa::foundation::{NSRect, NSSize, NSPoint};
use self::cocoa::base::id as cocoa_id;
use objc::runtime::{Class, Object};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;

lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}

#[repr(C)]
pub struct LinearLayout {
    base: CocoaControlBase,
    orientation: layout::Orientation,
    children: Vec<Box<UiControl>>,
}

impl LinearLayout {
    pub fn new(orientation: layout::Orientation) -> Box<LinearLayout> {
        Box::new(LinearLayout {
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
                 })
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

impl UiLayable for LinearLayout {
	fn layout_width(&self) -> layout::Size {
    	self.base.control_base.layout.width
    }
	fn layout_height(&self) -> layout::Size {
		self.base.control_base.layout.height
	}
	fn layout_gravity(&self) -> layout::Gravity {
		self.base.control_base.layout.gravity
	}
	fn layout_orientation(&self) -> layout::Orientation {
		self.base.control_base.layout.orientation
	}
	fn layout_alignment(&self) -> layout::Alignment {
		self.base.control_base.layout.alignment
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
	fn set_layout_orientation(&mut self, orientation: layout::Orientation) {
		self.base.control_base.layout.orientation = orientation;
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
    fn on_added_to_container(&mut self, parent: &UiContainer, x: u16, y: u16) {
    	use plygui_api::development::UiDrawable;
    	
        let (pw, ph) = parent.size();
        let (w, h, _) = self.measure(pw, ph);

        let rect = NSRect::new(NSPoint::new(x as f64, (ph - y - h) as f64),
                               NSSize::new(w as f64, h as f64));

        unsafe {
        	let base: cocoa_id = msg_send![WINDOW_CLASS.0, alloc];
	        let base: cocoa_id = msg_send![base, initWithFrame: rect];
			self.base.coords = Some((x as i32, y as i32));
	        self.base.control = msg_send![base, autorelease];
	        (&mut *self.base.control).set_ivar(IVAR, self as *mut _ as *mut ::std::os::raw::c_void);
	
	        let mut x = 0;
	        let mut y = 0;
	        let ll2: &LinearLayout = mem::transmute(self as *mut _ as *mut ::std::os::raw::c_void);
	        for ref mut child in self.children.as_mut_slice() {
	            let (xx, yy) = child.size();
	            match self.orientation {
	                layout::Orientation::Horizontal => {
	                    child.on_added_to_container(ll2, x, y);
	                    x += xx;
	                }
	                layout::Orientation::Vertical => {
	                    child.on_added_to_container(ll2, x, y);
	                    y += yy;
	                }
	            }
	            ll2.base.control.addSubview_(child.native_id() as cocoa_id);
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
    fn as_layable(&self) -> &UiLayable {
    	self
    }
	fn as_layable_mut(&mut self) -> &mut UiLayable {
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
            let (x, y) = {
                let mut x = 0;
                let mut y = 0;
                for ref child in self.children.as_slice() {
                    let (xx, yy) = child.size();
                    match self.orientation {
                        layout::Orientation::Horizontal => x += xx,
                        layout::Orientation::Vertical => y += yy,
                    }
                }
                (x, y)
            };
            unsafe {
                //let mut wc = common::cast_uicontrol_to_cocoa_mut(&mut new);
                let (_,yy) = new.size();
                match self.orientation {
                    layout::Orientation::Horizontal => {
                        new.on_added_to_container(self, x, y); //TODO padding
                    }
                    layout::Orientation::Vertical => {
                        let my_h = self.size().1;
                        new.on_added_to_container(self, x, my_h - y - yy); //TODO padding
                    }
                }
                self.base.control.addSubview_(new.native_id() as cocoa_id);
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

impl UiLinearLayout for LinearLayout {
    fn orientation(&self) -> layout::Orientation {
        self.orientation
    }
    fn set_orientation(&mut self, orientation: layout::Orientation) {
        self.orientation = orientation;
    }
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
}

impl development::UiDrawable for LinearLayout {
	fn draw(&mut self, coords: Option<(i32, i32)>) {
    	if coords.is_some() {
    		self.base.coords = coords;
    	}
    	if let Some((x, y)) = self.base.coords {
    		let (_, ph) = self.parent().unwrap().as_ref().size();
	        unsafe {
	        	let mut frame: NSRect = msg_send![self.base.control, frame];
	            frame.size = NSSize::new(self.base.measured_size.0 as f64,
	                                     self.base.measured_size.1 as f64);
	            frame.origin = NSPoint::new(x as f64, (ph as i32 - y - self.base.measured_size.1 as i32) as f64);
	            msg_send![self.base.control, setFrame: frame];
	        }
	        
	        let mut x = 0;
	        let mut y = 0;
	        
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
    	
    	let old_size = self.base.measured_size;
        self.base.measured_size = match self.visibility() {
        	types::Visibility::Gone => (0,0),
        	_ => {
        		let mut w = parent_width;
		        let mut h = parent_height;
		
		        if let layout::Size::Exact(ew) = self.layout_width() {
		            w = ew;
		        }
		        if let layout::Size::Exact(eh) = self.layout_height() {
		            w = eh;
		        }
		        let (mut ww, mut wm, mut hh, mut hm) = (0, 0, 0, 0);
		        for ref mut child in self.children.as_mut_slice() {
                    let (cw, ch, _) = child.measure(w, h);
                    ww += cw;
                    hh += ch;
                    wm = max(wm, cw);
                    hm = max(hm, ch);
                }
		        
		        match self.orientation {
		            layout::Orientation::Vertical => {
		                if let layout::Size::WrapContent = self.layout_height() {
		                    h = hh;
		                } 
		                if let layout::Size::WrapContent = self.layout_width() {
		                    w = wm;
		                }
		            }
		            layout::Orientation::Horizontal => {
		                if let layout::Size::WrapContent = self.layout_height() {
		                    h = hm;
		                }
		                if let layout::Size::WrapContent = self.layout_width() {
		                    w = ww;
		                }
		            }
		        }
		        (w, h)
        	}
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
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