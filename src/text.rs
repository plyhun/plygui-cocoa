use crate::common::{self, *};

use std::os::raw::c_char;

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiText", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const BASE_CLASS: &str = "NSTextView";

pub type Text = Member<Control<CocoaText>>;

#[repr(C)]
pub struct CocoaText {
    base: common::CocoaControlBase<Text>,
}

impl TextInner for CocoaText {
    fn with_text(text: &str) -> Box<Text> {
        use plygui_api::controls::HasLabel;

        let mut b = Box::new(Member::with_inner(
            Control::with_inner(
                CocoaText {
                    base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = b.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *b.as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr);
            let () = msg_send![b.as_inner_mut().as_inner_mut().base.control, setDrawsBackground: NO];
            let () = msg_send![b.as_inner_mut().as_inner_mut().base.control, setEditable: NO];
            let () = msg_send![b.as_inner_mut().as_inner_mut().base.control, setSelectable: NO];
        }
        b.set_label(text.into());
        b
    }
}

impl HasLabelInner for CocoaText {
    fn label(&self, _: &MemberBase) -> Cow<str> {
        unsafe {
            let label: cocoa_id = msg_send![self.base.control, title];
            let label: *const c_void = msg_send![label, UTF8String];
            ffi::CStr::from_ptr(label as *const c_char).to_string_lossy()
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: Cow<str>) {
        unsafe {
            let title = NSString::alloc(cocoa::base::nil).init_str(&label);
            let () = msg_send![self.base.control, setString: title];
            let () = msg_send![title, release];
        }
    }
}

impl ControlInner for CocoaText {
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
        use plygui_api::markup::MEMBER_TYPE_TEXT;

        fill_from_markup_base!(self, base, markup, registry, Text, [MEMBER_TYPE_TEXT]);
        fill_from_markup_label!(self, base, markup);
    }
}

impl HasNativeIdInner for CocoaText {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaText {
    fn on_size_set(&mut self, base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        use plygui_api::controls::HasLayout;

        let this = base.as_any_mut().downcast_mut::<Text>().unwrap();
        this.set_layout_width(layout::Size::Exact(width));
        this.set_layout_width(layout::Size::Exact(height));
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaText {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaText {}

impl HasLayoutInner for CocoaText {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for CocoaText {
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
                        label_size = common::measure_nsstring(msg_send![self.base.control, string]);
                        label_size.0 as i32
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.1 < 1 {
                            label_size = common::measure_nsstring(msg_send![self.base.control, string]);
                        }
                        label_size.1 as i32
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

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<dyn controls::Control> {
    Text::empty().into_control()
}

extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<Text>(this).unwrap();
        let () = msg_send![super(sp.as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        sp.call_on_size(param.width as u16, param.height as u16)
    }
}
default_impls_as!(Text);
