use crate::common::{self, *};

use cocoa::appkit::NSBezelStyle;

use std::os::raw::c_char;

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiButton", BASE_CLASS, |decl| {
            decl.add_method(sel!(mouseDown:), button_left_click as extern "C" fn(&mut Object, Sel, cocoa_id));
            decl.add_method(sel!(rightMouseDown:), button_right_click as extern "C" fn(&mut Object, Sel, cocoa_id));
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const BASE_CLASS: &str = "NSButton";

pub type Button = AMember<AControl<AButton<CocoaButton>>>;

#[repr(C)]
pub struct CocoaButton {
    base: common::CocoaControlBase<Button>,

    h_left_clicked: Option<callbacks::OnClick>,
    h_right_clicked: Option<callbacks::OnClick>,
    skip_callbacks: bool,
}

impl<O: controls::Button> NewButtonInner<O> for CocoaButton {
    fn with_uninit(ptr: &mut mem::MaybeUninit<O>) -> Self {
        let b = CocoaButton {
            base: common::CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>),
            h_left_clicked: None,
            h_right_clicked: None,
            skip_callbacks: false,
        };
        let selfptr = ptr as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *b.base.control).set_ivar(common::IVAR, selfptr);
            let () = msg_send![b.base.control, setBezelStyle: NSBezelStyle::NSSmallSquareBezelStyle];
        }
        b
    }
}

impl ButtonInner for CocoaButton {
    fn with_label<S: AsRef<str>>(label: S) -> Box<dyn controls::Button> {
        let mut b: Box<mem::MaybeUninit<Button>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AButton::with_inner(
                    <Self as NewButtonInner<Button>>::with_uninit(b.as_mut())
                )
            ),
        );
        let mut ab = unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        };
        controls::HasLabel::set_label(ab.as_mut(), label.as_ref().into());
        ab
    }
}

impl HasLabelInner for CocoaButton {
    fn label(&self, _: &MemberBase) -> Cow<str> {
        unsafe {
            let title: cocoa_id = msg_send![self.base.control, title];
            let title: *const c_void = msg_send![title, UTF8String];
            ffi::CStr::from_ptr(title as *const c_char).to_string_lossy()
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: Cow<str>) {
        unsafe {
            let title = NSString::alloc(cocoa::base::nil).init_str(&label);
            let () = msg_send![self.base.control, setTitle: title];
            let () = msg_send![title, release];
        }
    }
}

impl ClickableInner for CocoaButton {
    fn on_click(&mut self, cb: Option<callbacks::OnClick>) {
        self.h_left_clicked = cb;
    }
    fn click(&mut self, skip_callbacks: bool) {
        self.skip_callbacks = skip_callbacks;
        let () = unsafe { msg_send![self.base.control, mouseDown:nil] };
    }
}

impl ControlInner for CocoaButton {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        self.base.invalidate();
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        unsafe {
            self.base.on_removed_from_container();
        }
        self.base.invalidate();
    }

    fn parent(&self) -> Option<&dyn controls::Member> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&dyn controls::Member> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.root_mut()
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberBase, _control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        use plygui_api::markup::MEMBER_TYPE_BUTTON;

        fill_from_markup_base!(self, base, markup, registry, Button, [MEMBER_TYPE_BUTTON]);
        fill_from_markup_label!(self, base, markup);
        fill_from_markup_callbacks!(self, markup, registry, [on_click => plygui_api::callbacks::OnClick]);
    }
}

impl HasNativeIdInner for CocoaButton {
    type Id = common::CocoaId;

    fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaButton {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaButton {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaButton {}

impl HasLayoutInner for CocoaButton {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for CocoaButton {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
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
                (cmp::max(0, w) as u16, cmp::max(0, h) as u16)
            },
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}
impl Spawnable for CocoaButton {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_label("").into_control()
    }
}

extern "C" fn button_left_click(this: &mut Object, _: Sel, param: cocoa_id) {
    unsafe {
        let button = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
        let () = msg_send![super(button.inner_mut().inner_mut().inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), mouseDown: param];
        if !button.inner().inner().inner().skip_callbacks {
            if let Some(ref mut cb) = button.inner_mut().inner_mut().inner_mut().h_left_clicked {
            let b2 = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
                (cb.as_mut())(b2);
            } 
        }
    }
}
extern "C" fn button_right_click(this: &mut Object, _: Sel, param: cocoa_id) {
    //println!("right!");
    unsafe {
        let button = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
        if let Some(ref mut cb) = button.inner_mut().inner_mut().inner_mut().h_right_clicked {
            let b2 = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
            (cb.as_mut())(b2);
        }
        let () = msg_send![super(button.inner_mut().inner_mut().inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), rightMouseDown: param];
    }
}
extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
        let b2 = common::member_from_cocoa_id_mut::<Button>(this).unwrap();
        (b.inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: controls::Button>(this: &mut Button, _: Sel, param: NSSize) {
    unsafe {
        let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        this.call_on_size::<O>(param.width as u16, param.height as u16)
    }
}
