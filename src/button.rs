use super::*;

use plygui_api::{layout, types, development, callbacks};
use plygui_api::traits::{UiControl, UiLayable, UiButton, UiMember, UiContainer};
use plygui_api::members::MEMBER_ID_BUTTON;

use self::cocoa::appkit::{NSBezelStyle, NSButton};
use self::cocoa::foundation::{NSString, NSRect, NSSize, NSPoint};
use self::cocoa::base::id;
use objc::runtime::{Class, Object, Sel};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;

pub const IVAR: &str = MEMBER_ID_BUTTON;
lazy_static! {
	static ref WINDOW_CLASS: common::RefClass = unsafe { register_window_class() };
}

const PADDING: u16 = 15;
const BASE_CLASS: &str = "NSButton";

#[repr(C)]
pub struct Button {
    base: common::CocoaControlBase,

    label: String,
    h_left_clicked: Option<callbacks::Click>,
    h_right_clicked: Option<callbacks::Click>,
}

impl Button {
    pub fn new(label: &str) -> Box<Button> {
        Box::new(Button {
                     base: common::CocoaControlBase::with_params(
                             	development::UiMemberFunctions {
		                             fn_member_id: member_id,
								     fn_is_control: is_control,
								     fn_is_control_mut: is_control_mut,
								     fn_size: size,
	                             },
                             ),
                     label: label.to_owned(),
                     h_left_clicked: None,
                     h_right_clicked: None,
                 })
    }
}

impl UiButton for Button {
    fn label(&self) -> &str {
        self.label.as_ref()
    }
    fn set_label(&mut self, label: &str) {
    	self.label = label.into();
    	if self.base.control != 0 as id {
    		unsafe {
    			let title = NSString::alloc(cocoa::base::nil).init_str(self.label.as_ref());
	    		self.base.control.setTitle_(title);
    		}
    	}
    }
    fn on_left_click(&mut self, cb: Option<callbacks::Click>) {
        self.h_left_clicked = cb;
    }
    /*fn on_right_click(&mut self, cb: Option<Box<FnMut(&mut UiButton)>>) {
        self.h_right_clicked = cb;
    }*/
    
    fn as_control(&self) -> &UiControl {
    	self
    }
	fn as_control_mut(&mut self) -> &mut UiControl {
		self
	}
}

impl UiLayable for Button {
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

impl UiControl for Button {
    fn is_container_mut(&mut self) -> Option<&mut UiContainer> {
        None
    }
    fn is_container(&self) -> Option<&UiContainer> {
        None
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
    fn on_added_to_container(&mut self, parent: &UiContainer, x: u16, y: u16) {
    	use plygui_api::development::UiDrawable;
    	
        let (pw, ph) = parent.size();
        let (w, h, _) = self.measure(pw, ph);

        let rect = NSRect::new(NSPoint::new(x as f64, (ph - y - h) as f64),
                               NSSize::new(w as f64, h as f64));

        unsafe {
	        let base: id = msg_send![WINDOW_CLASS.0, alloc];
	        let base: id = msg_send![base, initWithFrame: rect];
	
	        self.base.control = msg_send![base, autorelease];

        	let title = NSString::alloc(cocoa::base::nil).init_str(self.label.as_ref());
	        self.base.control.setTitle_(title);
	        self.base
	            .control
	            .setBezelStyle_(NSBezelStyle::NSSmallSquareBezelStyle);
	
	        (&mut *self.base.control).set_ivar(IVAR, self as *mut _ as *mut ::std::os::raw::c_void);
	        msg_send![title, release];
        }
    }
    fn on_removed_from_container(&mut self, _: &UiContainer) {
        unsafe { self.base.on_removed_from_container(); }
    }	
    
    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
    	use plygui_api::markup::MEMBER_TYPE_BUTTON;
    	
    	fill_from_markup_base!(self, markup, registry, Button, [MEMBER_ID_BUTTON, MEMBER_TYPE_BUTTON]);
    	fill_from_markup_label!(self, markup);
    	//fill_from_markup_callbacks!(self, markup, registry, ["on_left_click" => FnMut(&mut UiButton)]);
    	
    	if let Some(on_left_click) = markup.attributes.get("on_left_click") {
    		let callback: callbacks::Click = registry.pop_callback(on_left_click.as_attribute()).unwrap();
    		self.on_left_click(Some(callback));
    	}
    }
    fn as_layable(&self) -> &UiLayable {
    	self
    }
	fn as_layable_mut(&mut self) -> &mut UiLayable {
		self
	}
}

impl UiMember for Button {
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
    fn is_control(&self) -> Option<&UiControl> {
    	Some(self)
    }
    fn is_control_mut(&mut self) -> Option<&mut UiControl> {
    	Some(self)
    } 
    fn as_base(&self) -> &types::UiMemberBase {
    	self.base.control_base.member_base.as_ref()
    }
    fn as_base_mut(&mut self) -> &mut types::UiMemberBase {
    	self.base.control_base.member_base.as_mut()
    }
}

impl development::UiDrawable for Button {
	fn draw(&mut self, coords: Option<(i32, i32)>) {
    	if coords.is_some() {
    		self.base.coords = coords;
    	}
    	unsafe {
            let mut frame: NSRect = msg_send![self.base.control, frame];
            frame.size = NSSize::new(self.base.measured_size.0 as f64,
                                     self.base.measured_size.1 as f64);
            let (x, y) = self.base.coords.unwrap();
            frame.origin = NSPoint::new(x as f64, y as f64);
            msg_send![self.base.control, setFrame: frame];

            if let Some(ref mut cb) = self.base.h_resize {
                let object: &Object = mem::transmute(self.base.control);
                let saved: *mut c_void = *object.get_ivar(IVAR);
                let mut button2: &mut Button = mem::transmute(saved);
                (cb.as_mut())(button2,
                     self.base.measured_size.0,
                     self.base.measured_size.1);
            }
        }
    }
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	let old_size = self.base.measured_size;
        self.base.measured_size = match self.visibility() {
            types::Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size = (0, 0);
                let w = match self.base.control_base.layout.width {
                    layout::Size::MatchParent => parent_width,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => {
                        if label_size.0 < 1 {
                            label_size = common::measure_string(self.label.as_ref());
                            label_size.0 += PADDING;
                            label_size.1 += PADDING;
                        }
                        label_size.0 as u16
                    } 
                };
                let h = match self.base.control_base.layout.height {
                    layout::Size::MatchParent => parent_height,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => {
                        if label_size.1 < 1 {
                            label_size = common::measure_string(self.label.as_ref());
                            label_size.0 += PADDING;
                            label_size.1 += PADDING;
                        }
                        label_size.1 as u16
                    } 
                };
                (w, h)
            },
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<UiControl> {
	Button::new("")
}

unsafe fn register_window_class() -> common::RefClass {
    let superclass = Class::get(BASE_CLASS).unwrap();
    let mut decl = ClassDecl::new(IVAR, superclass).unwrap();

    decl.add_method(sel!(mouseDown:),
                    button_left_click as extern "C" fn(&Object, Sel, id));
    decl.add_method(sel!(rightMouseDown:),
                    button_right_click as extern "C" fn(&Object, Sel, id));
    decl.add_ivar::<*mut c_void>(IVAR);

    common::RefClass(decl.register())
}

extern "C" fn button_left_click(this: &Object, _: Sel, param: id) {
	unsafe {
        let saved: *mut c_void = *this.get_ivar(IVAR);
        let button: &mut Button = mem::transmute(saved.clone());
        msg_send![super(button.base.control, Class::get(BASE_CLASS).unwrap()), mouseDown: param];
        if let Some(ref mut cb) = button.h_left_clicked {
            let b2: &mut Button = mem::transmute(saved);
            (cb.as_mut())(b2);
        }
    }
}
extern "C" fn button_right_click(this: &Object, _: Sel, param: id) {
    //println!("right!");
    unsafe {
        let saved: *mut c_void = *this.get_ivar(IVAR);
        let button: &mut Button = mem::transmute(saved.clone());
        if let Some(ref mut cb) = button.h_right_clicked {
            let b2: &mut Button = mem::transmute(saved);
            (cb.as_mut())(b2);
        }
        msg_send![super(button.base.control, Class::get(BASE_CLASS).unwrap()), rightMouseDown: param];
    }
}

impl_is_control!(Button);
impl_size!(Button);
impl_member_id!(MEMBER_ID_BUTTON);