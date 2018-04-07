use super::*;
use super::common::*;

use plygui_api::{layout, ids, types, development, callbacks};
use plygui_api::traits::{UiControl, UiHasLayout, UiHasLabel, UiSingleContainer, UiFrame, UiMember, UiContainer};
use plygui_api::members::MEMBER_ID_FRAME;

use self::cocoa::appkit::NSView;
use self::cocoa::foundation::{NSString, NSRect, NSSize, NSPoint};
use self::cocoa::base::id as cocoa_id;
use objc::runtime::{Class, Object};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;
use std::borrow::Cow;

lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}

#[repr(C)]
pub struct Frame {
    base: CocoaControlBase,
    label: String,
    child: Option<Box<UiControl>>,
}

impl Frame {
    pub fn new(label: &str) -> Box<Frame> {
        Box::new(Frame {
                     base: common::CocoaControlBase::with_params(
		                     	invalidate_impl,
                             	 development::UiMemberFunctions {
		                             fn_member_id: member_id,
								     fn_is_control: is_control,
								     fn_is_control_mut: is_control_mut,
								     fn_size: size,
	                             },
                             ),
                     label: label.into(),
                     child: None,
                 })
    }
}
impl UiMember for Frame {
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

impl UiHasLayout for Frame {
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

impl UiControl for Frame {
    fn on_added_to_container(&mut self, parent: &UiContainer, x: i32, y: i32) {
    	use plygui_api::development::UiDrawable;
    	
        let (pw, ph) = parent.draw_area_size();
        let (lm, tm, _, _) = self.base.control_base.layout.margin.into();
        let (w, h, _) = self.measure(pw, ph);

        let rect = NSRect::new(NSPoint::new(x as f64, (ph as i32 - y - h as i32) as f64),
                               NSSize::new(w as f64, h as f64));

        unsafe {
        	let base: cocoa_id = msg_send![WINDOW_CLASS.0, alloc];
	        let base: cocoa_id = msg_send![base, initWithFrame: rect];
			self.base.coords = Some((x as i32, y as i32));
	        self.base.control = msg_send![base, autorelease];
	        (&mut *self.base.control).set_ivar(IVAR, self as *mut _ as *mut ::std::os::raw::c_void);
	
	        let frame2: &Frame = mem::transmute(self as *mut _ as *mut ::std::os::raw::c_void);
	        if let Some(ref mut child) = self.child {
	        	let (lp, tp, _, _) = self.base.control_base.layout.padding.into();
		        frame2.base.control.addSubview_(child.native_id() as cocoa_id);
	        	child.on_added_to_container(frame2, lm + lp, tm + tp);
	        }
        }
    }
    fn on_removed_from_container(&mut self, _: &UiContainer) {
        let frame2: &Frame = unsafe { mem::transmute(self as *mut _ as *mut ::std::os::raw::c_void) };
        if let Some(ref mut child) = self.child {
            child.on_removed_from_container(frame2);
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
    	
    	fill_from_markup_base!(self, markup, registry, Frame, [MEMBER_ID_LAYOUT_LINEAR, MEMBER_TYPE_LINEAR_LAYOUT]);
		fill_from_markup_children!(self, markup, registry);		
    }
    fn as_has_layout(&self) -> &UiHasLayout {
    	self
    }
	fn as_has_layout_mut(&mut self) -> &mut UiHasLayout {
		self
	}
}

impl UiContainer for Frame {
    fn is_single_mut(&mut self) -> Option<&mut UiSingleContainer> {
        Some(self)
    }
    fn is_single(&self) -> Option<&UiSingleContainer> {
        Some(self)
    }
    fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut UiControl> {
        if id_ == self.base.control_base.member_base.id {
			return Some(self)
		}
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id_);
            }
        }
        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&UiControl> {
        if id_ == self.base.control_base.member_base.id {
			return Some(self)
		}
        if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id_);
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

impl UiHasLabel for Frame {
	fn label<'a>(&'a self) -> Cow<'a,str> {
        Cow::Borrowed(self.label.as_ref())
    }
    fn set_label(&mut self, label: &str) {
    	self.label = label.into();
    	if self.base.control != 0 as cocoa_id {
    		unsafe {
    			let title = NSString::alloc(cocoa::base::nil).init_str(self.label.as_ref());
    			msg_send![self.base.control, setTitle: title];
    			msg_send![title, release];
	    	}
    	}
    }
}

impl UiSingleContainer for Frame {
	fn set_child(&mut self, child: Option<Box<UiControl>>) -> Option<Box<UiControl>> {
        let old = self.child.take();
        self.child = child;
        old
    }
    fn child(&self) -> Option<&UiControl> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut UiControl> {
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
    fn as_container(&self) -> &UiContainer {
    	self
    }
	fn as_container_mut(&mut self) -> &mut UiContainer {
		self
	}
}

impl UiFrame for Frame {
    fn as_control(&self) -> &UiControl {
	    	self
    }
	fn as_control_mut(&mut self) -> &mut UiControl {
		self
	}
	fn as_single_container(&self) -> &UiSingleContainer {
		self
	}
	fn as_single_container_mut(&mut self) -> &mut UiSingleContainer {
		self
	}
	fn as_has_label(&self) -> &UiHasLabel {
		self
	}
	fn as_has_label_mut(&mut self) -> &mut UiHasLabel {
		self
	}
}

impl development::UiDrawable for Frame {
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
	        
	        if let Some(ref mut child) = self.child {
	            child.draw(Some((x, y)));  
	        }    	
	        if let Some(ref mut cb) = self.base.h_resize {
	            unsafe {
	                let object: &Object = mem::transmute(self.base.control);
	                let saved: *mut c_void = *object.get_ivar(IVAR);
	                let mut frame2: &mut Frame = mem::transmute(saved);
	                (cb.as_mut())(frame2, self.base.measured_size.0, self.base.measured_size.1);
	            }
	        }
	    }
    }
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	use std::cmp::max;
    	
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
	        			if let Some(ref mut child) =  self.child {
		                    let (cw, _, _) = child.measure(
		                    	max(0, parent_width as i32 - hp) as u16, 
		                    	max(0, parent_height as i32 - vp) as u16
		                    );
		                    w += cw as i32;
		                    measured = true;
		                }
	        			max(0, w as i32 + hp) as u16
        			}
        		};
        		let h = match self.layout_height() {
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
		                    h += ch as i32;
		                    /*let mut label_size: windef::SIZE = unsafe { mem::zeroed() };
			        		let label = label_size = common::measure_string(self.label.as_ref());
                            label_size.0 += PADDING;
                            label_size.1 += PADDING
	                        self.label_padding = label_size.cy as i32;
	                        h += self.label_padding;*/
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
	Frame::new("")
}

unsafe fn register_window_class() -> RefClass {
    let superclass = Class::get("NSBox").unwrap();
    let mut decl = ClassDecl::new(MEMBER_ID_FRAME, superclass).unwrap();

    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}
impl_invalidate!(Frame);
impl_is_control!(Frame);
impl_size!(Frame);
impl_member_id!(MEMBER_ID_FRAME);
