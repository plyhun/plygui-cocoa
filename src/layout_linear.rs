use super::*;
use super::common::*;

use self::cocoa::appkit::NSView;
use self::cocoa::foundation::{NSRect, NSSize, NSPoint};
use self::cocoa::base::id as cocoa_id;
use objc::runtime::{Class, Object};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;

pub const IVAR: &str = development::CLASS_ID_LAYOUT_LINEAR;
lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}

use {development, layout, Id, UiRole, UiRoleMut, UiControl, UiMember, UiContainer, UiMultiContainer, UiLinearLayout, Visibility};

#[repr(C)]
pub struct LinearLayout {
    base: CocoaControlBase,
    orientation: layout::Orientation,
    children: Vec<Box<UiControl>>,
}

impl LinearLayout {
    pub fn new(orientation: layout::Orientation) -> Box<LinearLayout> {
        Box::new(LinearLayout {
                     base: Default::default(),
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
    fn set_visibility(&mut self, visibility: Visibility) {
        self.base.set_visibility(visibility);
    }
    fn visibility(&self) -> Visibility {
        self.base.visibility()
    }
    fn size(&self) -> (u16, u16) {
        self.base.measured_size
    }

    fn on_resize(&mut self, handler: Option<Box<FnMut(&mut UiMember, u16, u16)>>) {
        self.base.h_resize = handler;
    }

    fn role<'a>(&'a self) -> UiRole<'a> {
        UiRole::LinearLayout(self)
    }
    fn role_mut<'a>(&'a mut self) -> UiRoleMut<'a> {
        UiRoleMut::LinearLayout(self)
    }
    fn native_id(&self) -> NativeId {
        self.base.control
    }
    fn id(&self) -> Id {
    	self.base.id()
    }
}

impl UiControl for LinearLayout {
    fn layout_width(&self) -> layout::Size {
    	self.base.layout_width()
    }
	fn layout_height(&self) -> layout::Size {
		self.base.layout_height()
	}
	fn layout_gravity(&self) -> layout::Gravity {
		self.base.layout_gravity()
	}
	fn layout_orientation(&self) -> layout::Orientation {
		self.base.layout_orientation()
	}
	fn layout_alignment(&self) -> layout::Alignment {
		self.base.layout_alignment()
	}
	
	fn set_layout_width(&mut self, width: layout::Size) {
		self.base.set_layout_width(width);
	}
	fn set_layout_height(&mut self, height: layout::Size) {
		self.base.set_layout_height(height);
	}
	fn set_layout_gravity(&mut self, gravity: layout::Gravity) {
		self.base.set_layout_gravity(gravity);
	}
	fn set_layout_orientation(&mut self, orientation: layout::Orientation) {
		self.base.set_layout_orientation(orientation);
	}
	fn set_layout_alignment(&mut self, alignment: layout::Alignment) {
		self.base.set_layout_alignment(alignment);
	}
    fn draw(&mut self, coords: Option<(i32, i32)>) {
    	if coords.is_some() {
    		self.base.coords = coords;
    	}
    	if let Some((x, y)) = self.base.coords {
	        unsafe {
	        	let mut frame: NSRect = msg_send![self.base.control, frame];
	            frame.size = NSSize::new(self.base.measured_size.0 as f64,
	                                     self.base.measured_size.1 as f64);
	            frame.origin = NSPoint::new(x as f64, y as f64);
	            msg_send![self.base.control, setFrame: frame];
	        }
	        
	        let mut x = 0;
	        let mut y = 0;
	        let my_h = self.size().1;
	        println!("draw {} at {}/{}", my_h, x, y);
	        for mut child in self.children.as_mut_slice() {
	            let child_size = child.size();
	            child.draw(Some((x, my_h as i32 - y - child_size.1 as i32)));      
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
	                (cb)(ll2, self.base.measured_size.0, self.base.measured_size.1);
	            }
	        }
	    }
    }
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	let old_size = self.base.measured_size;
        self.base.measured_size = match self.base.visibility() {
	        Visibility::Gone => (0,0),
	        _ => {
	        	let mut w = parent_width;
		        let mut h = parent_height;
		
		        if let layout::Size::Exact(ew) = self.base.layout_width() {
		            w = ew;
		        }
		        if let layout::Size::Exact(eh) = self.base.layout_height() {
		            w = eh;
		        }
		        if w == parent_width || h == parent_height {
		            let mut ww = 0;
		            let mut hh = 0;
		            for ref mut child in self.children.as_mut_slice() {
		                let (cw, ch,_) = child.measure(w, h);
		                ww += cw;
		                hh += ch;
		            }
		            match self.orientation {
		                layout::Orientation::Vertical => {
		                    if let layout::Size::WrapContent = self.base.layout_height() {
		                        h = hh;
		                    }
		                }
		                layout::Orientation::Horizontal => {
		                    if let layout::Size::WrapContent = self.base.layout_width() {
		                        w = ww;
		                    }
		                }
		            }
		        }
		        (w, h)
	        }
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn is_container_mut(&mut self) -> Option<&mut UiContainer> {
        Some(self)
    }
    fn is_container(&self) -> Option<&UiContainer> {
        Some(self)
    }
    fn parent(&self) -> Option<&UiContainer> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut UiContainer> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&UiContainer> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut UiContainer> {
        self.base.root_mut()
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
                let mut wc = common::cast_uicontrol_to_cocoa_mut(&mut new);
                let (xx,yy) = wc.size();
                match self.orientation {
                    layout::Orientation::Horizontal => {
                        wc.on_added_to_container(self, x, y); //TODO padding
                    }
                    layout::Orientation::Vertical => {
                        let my_h = self.size().1;
                        wc.on_added_to_container(self, x, my_h - y - yy); //TODO padding
                    }
                }
                self.base.control.addSubview_(wc.as_base().control);
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
        unsafe {
            let mut wc = common::cast_uicontrol_to_cocoa_mut(&mut child);
            wc.on_removed_from_container(self);
        }

        Some(child)
    }
    fn child_at(&self, index: usize) -> Option<&Box<UiControl>> {
        self.children.get(index)
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut Box<UiControl>> {
        self.children.get_mut(index)
    }
}

impl UiContainer for LinearLayout {
    fn set_child(&mut self, child: Option<Box<UiControl>>) -> Option<Box<UiControl>> {
        let old = self.children.pop();
        self.children.clear();

        if let Some(child) = child {
            self.set_child_to(0, child);
        }

        old
    }
    fn child(&self) -> Option<&UiControl> {
        self.children.get(0).map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut UiControl> {
        //self.children.get_mut(0).map(|c|c.as_mut()) // WTF??
        if self.children.len() > 0 {
            Some(self.children[0].as_mut())
        } else {
            None
        }
    }
    fn find_control_by_id_mut(&mut self, id_: Id) -> Option<&mut UiControl> {
        if self.id() == id_ {
            return Some(self);
        }
        for child in self.children.as_mut_slice() {
            if child.id() == id_ {
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
    fn find_control_by_id(&self, id_: Id) -> Option<&UiControl> {
        if self.id() == id_ {
            return Some(self);
        }
        for child in self.children.as_slice() {
            if child.id() == id_ {
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
    fn is_multi_mut(&mut self) -> Option<&mut UiMultiContainer> {
        Some(self)
    }
    fn is_multi(&self) -> Option<&UiMultiContainer> {
        Some(self)
    }
}

impl UiLinearLayout for LinearLayout {
    fn orientation(&self) -> layout::Orientation {
        self.orientation
    }
    fn set_orientation(&mut self, orientation: layout::Orientation) {
        self.orientation = orientation;
    }
}

unsafe impl CocoaContainer for LinearLayout {
    unsafe fn cocoa_id(&self) -> cocoa_id {
        self.base.control
    }
}

unsafe impl CocoaControl for LinearLayout {
    unsafe fn on_added_to_container(&mut self, parent: &common::CocoaContainer, x: u16, y: u16) {
        let (pw, ph) = parent.size();
        let (w, h, _) = self.measure(pw, ph);

        let rect = NSRect::new(NSPoint::new(x as f64, (ph - y - h) as f64),
                               NSSize::new(w as f64, h as f64));

        let base: cocoa_id = msg_send![WINDOW_CLASS.0, alloc];
        let base: cocoa_id = msg_send![base, initWithFrame: rect];

        self.base.control = msg_send![base, autorelease];
        (&mut *self.base.control).set_ivar(IVAR, self as *mut _ as *mut ::std::os::raw::c_void);

        let mut x = 0;
        let mut y = 0;
        let ll2: &LinearLayout = mem::transmute(self as *mut _ as *mut ::std::os::raw::c_void);
        for ref mut child in self.children.as_mut_slice() {
            let mut wc = common::cast_uicontrol_to_cocoa_mut(child);
            let (xx, yy) = wc.size();
            match self.orientation {
                layout::Orientation::Horizontal => {
                    wc.on_added_to_container(ll2, x, y);
                    x += xx;
                }
                layout::Orientation::Vertical => {
                    wc.on_added_to_container(ll2, x, y);
                    y += yy;
                }
            }
            ll2.base.control.addSubview_(wc.as_base().control);
        }
    }
    unsafe fn on_removed_from_container(&mut self, _: &common::CocoaContainer) {
        let ll2: &LinearLayout = mem::transmute(self as *mut _ as *mut ::std::os::raw::c_void);
        for ref mut child in self.children.as_mut_slice() {
            let mut wc = common::cast_uicontrol_to_cocoa_mut(child);
            wc.on_removed_from_container(ll2);
        }
        self.base.on_removed_from_container();
    }
    fn as_base(&self) -> &common::CocoaControlBase {
    	&self.base
    }
    fn as_base_mut(&mut self) -> &mut common::CocoaControlBase {
    	&mut self.base
    }
}

unsafe fn register_window_class() -> RefClass {
    let superclass = Class::get("NSView").unwrap();
    let mut decl = ClassDecl::new(IVAR, superclass).unwrap();

    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}
