use super::*;
use super::common::*;

use self::cocoa::appkit::{NSBezelStyle, NSButton};
use self::cocoa::foundation::{NSString, NSRect, NSSize, NSPoint};
use self::cocoa::base::id;
use objc::runtime::{Class, Object, Sel};
use objc::declare::ClassDecl;

use std::mem;
use std::os::raw::c_void;

pub const IVAR: &str = development::CLASS_ID_BUTTON;
lazy_static! {
	static ref WINDOW_CLASS: RefClass = unsafe { register_window_class() };
}

const PADDING: u16 = 15;
const BASE_CLASS: &str = "NSButton";

use {development, layout, Id, UiRole, UiRoleMut, UiControl, UiButton, UiMember, Visibility, UiContainer};

#[repr(C)]
pub struct Button {
    base: common::CocoaControlBase,

    label: String,
    h_left_clicked: Option<Box<FnMut(&mut UiButton)>>,
    h_right_clicked: Option<Box<FnMut(&mut UiButton)>>,
}

impl Button {
    pub fn new(label: &str) -> Box<Button> {
        Box::new(Button {
                     base: Default::default(),
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
    fn on_left_click(&mut self, cb: Option<Box<FnMut(&mut UiButton)>>) {
        self.h_left_clicked = cb;
    }
    /*fn on_right_click(&mut self, cb: Option<Box<FnMut(&mut UiButton)>>) {
        self.h_right_clicked = cb;
    }*/
}

impl UiControl for Button {
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
    fn is_container_mut(&mut self) -> Option<&mut UiContainer> {
        None
    }
    fn is_container(&self) -> Option<&UiContainer> {
        None
    }
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
                (cb)(button2,
                     self.base.measured_size.0,
                     self.base.measured_size.1);
            }
        }
    }
    fn measure(&mut self, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
    	let old_size = self.base.measured_size;
        self.base.measured_size = match self.visibility() {
            Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size = (0, 0);
                let w = match self.base.layout_width() {
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
                let h = match self.base.layout_height() {
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

impl UiMember for Button {
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
        UiRole::Button(self)
    }
    fn role_mut<'a>(&'a mut self) -> UiRoleMut<'a> {
        UiRoleMut::Button(self)
    }
    fn native_id(&self) -> NativeId {
        self.base.control
    }
    fn id(&self) -> Id {
    	self.base.id()
    }
    fn is_control(&self) -> Option<&UiControl> {
    	Some(self)
    }
    fn is_control_mut(&mut self) -> Option<&mut UiControl> {
    	Some(self)
    } 
}

unsafe impl common::CocoaControl for Button {
    unsafe fn on_added_to_container(&mut self, parent: &common::CocoaContainer, x: u16, y: u16) {
        let (pw, ph) = parent.size();
        let (w, h, _) = self.measure(pw, ph);

        let rect = NSRect::new(NSPoint::new(x as f64, (ph - y - h) as f64),
                               NSSize::new(w as f64, h as f64));

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
    unsafe fn on_removed_from_container(&mut self, _: &common::CocoaContainer) {
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
    let superclass = Class::get(BASE_CLASS).unwrap();
    let mut decl = ClassDecl::new(IVAR, superclass).unwrap();

    decl.add_method(sel!(mouseDown:),
                    button_left_click as extern "C" fn(&Object, Sel, id));
    decl.add_method(sel!(rightMouseDown:),
                    button_right_click as extern "C" fn(&Object, Sel, id));
    decl.add_ivar::<*mut c_void>(IVAR);

    RefClass(decl.register())
}

extern "C" fn button_left_click(this: &Object, _: Sel, param: id) {
	unsafe {
        let saved: *mut c_void = *this.get_ivar(IVAR);
        let button: &mut Button = mem::transmute(saved.clone());
        msg_send![super(button.base.control, Class::get(BASE_CLASS).unwrap()), mouseDown: param];
        if let Some(ref mut cb) = button.h_left_clicked {
            let b2: &mut Button = mem::transmute(saved);
            (cb)(b2);
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
            (cb)(b2);
        }
        msg_send![super(button.base.control, Class::get(BASE_CLASS).unwrap()), rightMouseDown: param];
    }
}
