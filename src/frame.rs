use crate::common::{self, *};

pub use std::os::raw::c_char;

const INNER_PADDING_H: i32 = 8; // TODO: WHY???
const INNER_PADDING_V: i32 = 8; // TODO: WHY???

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiFrame", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const BASE_CLASS: &str = "NSBox";

pub type Frame = AMember<AControl<AContainer<ASingleContainer<AFrame<CocoaFrame>>>>>;

#[repr(C)]
pub struct CocoaFrame {
    base: common::CocoaControlBase<Frame>,
    label_padding: (i32, i32),
    child: Option<Box<dyn controls::Control>>,
}
impl<O: controls::Frame> NewFrameInner<O> for CocoaFrame {
    fn with_uninit(ptr: &mut mem::MaybeUninit<O>) -> Self {
        let fr = CocoaFrame {
            base: common::CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>),
            label_padding: (0, 0),
            child: None,
        };
        let selfptr = ptr as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *fr.base.control).set_ivar(common::IVAR, selfptr);
        }
        fr
    }
}
impl FrameInner for CocoaFrame {
    fn with_label<S: AsRef<str>>(label: S) -> Box<dyn controls::Frame> {
        let mut b: Box<mem::MaybeUninit<Frame>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    ASingleContainer::with_inner(
                        AFrame::with_inner(
                            <Self as NewFrameInner<Frame>>::with_uninit(b.as_mut())
                        )
                    ),
                ),
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

impl CocoaFrame {
    fn measure_label(&mut self) {
        let label_size = unsafe { common::measure_nsstring(msg_send![self.base.control, title]) };
        self.label_padding = (label_size.0 as i32, label_size.1 as i32);
    }
}

impl SingleContainerInner for CocoaFrame {
    fn set_child(&mut self, _: &mut MemberBase, child: Option<Box<dyn controls::Control>>) -> Option<Box<dyn controls::Control>> {
        let mut old = self.child.take();
        self.child = child;
        if let Some(ref mut child) = self.child {
            unsafe {
                let child_id = child.native_id() as cocoa_id;
                (&mut *child_id).set_ivar(common::IVAR_PARENT, self.base.control as *mut c_void);
                let () = msg_send![self.base.control, addSubview: child_id];
                let frame2 = common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap();
                let (pw, ph) = frame2.inner().base.measured;
                if self.base.root().is_some() {
                    child.on_added_to_container(
                        frame2,
                        0,
                        INNER_PADDING_V + self.label_padding.1 as i32,
                        cmp::max(0, pw as i32 - INNER_PADDING_H - INNER_PADDING_H) as u16,
                        cmp::max(0, ph as i32 - INNER_PADDING_V - INNER_PADDING_V) as u16,
                    );
                }
            }
        }
        if let Some(ref mut old) = old {
            unsafe {
                let child_id = old.native_id() as cocoa_id;
                *(&mut *child_id).get_mut_ivar::<*mut c_void>(common::IVAR_PARENT) = ptr::null_mut();
                let () = msg_send![child_id, removeFromSuperview];
                let frame2 = common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap();
                if self.base.root().is_some() {
                    old.on_removed_from_container(frame2);
                }
            }
        }
        self.base.invalidate();
        old
    }
    fn child(&self) -> Option<&dyn controls::Control> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut dyn controls::Control> {
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
}

impl ContainerInner for CocoaFrame {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        if let Some(child) = self.child.as_mut() {
            match arg {
                types::FindBy::Id(id) => {
                    if child.as_member_mut().id() == id {
                        return Some(child.as_mut());
                    }
                }
                types::FindBy::Tag(tag) => {
                    if let Some(mytag) = child.as_member_mut().tag() {
                        if tag == mytag {
                            return Some(child.as_mut());
                        }
                    }
                }
            }
            if let Some(c) = child.is_container_mut() {
                c.find_control_mut(arg)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn find_control<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
        if let Some(child) = self.child.as_ref() {
            match arg {
                types::FindBy::Id(id) => {
                    if child.as_member().id() == id {
                        return Some(child.as_ref());
                    }
                }
                types::FindBy::Tag(tag) => {
                    if let Some(mytag) = child.as_member().tag() {
                        if tag == mytag {
                            return Some(child.as_ref());
                        }
                    }
                }
            }
            if let Some(c) = child.is_container() {
                c.find_control(arg)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl HasLabelInner for CocoaFrame {
    fn label(&self, _: &MemberBase) -> Cow<str> {
        unsafe {
            let label: cocoa_id = msg_send![self.base.control, getTitle];
            let label: *const c_void = msg_send![label, UTF8String];
            ffi::CStr::from_ptr(label as *const c_char).to_string_lossy()
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: Cow<str>) {
        unsafe {
            let title = NSString::alloc(cocoa::base::nil).init_str(&label);
            let () = msg_send![self.base.control, setTitle: title];
            let () = msg_send![title, release];
        }
        self.measure_label();
    }
}

impl ControlInner for CocoaFrame {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);

        if let Some(ref mut child) = self.child {
            let frame2 = unsafe { common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap() };
            let (pw, ph) = control.measured;
            child.on_added_to_container(
                frame2,
                0,
                INNER_PADDING_V + self.label_padding.1 as i32,
                cmp::max(0, pw as i32 - INNER_PADDING_H - INNER_PADDING_H) as u16,
                cmp::max(0, ph as i32 - INNER_PADDING_V - INNER_PADDING_V) as u16,
            );
        }
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        let frame2 = unsafe { common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap() };
        if let Some(ref mut child) = self.child {
            child.on_removed_from_container(frame2);
        }
        unsafe {
            self.base.on_removed_from_container();
        }
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
        use plygui_api::markup::MEMBER_TYPE_FRAME;

        fill_from_markup_base!(self, base, markup, registry, Frame, [MEMBER_TYPE_FRAME]);
        fill_from_markup_label!(self, base, markup);
        fill_from_markup_child!(self, base, markup, registry);
    }
}

impl HasNativeIdInner for CocoaFrame {
    type Id = common::CocoaId;

    fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaFrame {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaFrame {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaFrame {}

impl HasLayoutInner for CocoaFrame {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for CocoaFrame {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
        if let Some(ref mut child) = self.child {
            child.draw(Some((0, INNER_PADDING_V + self.label_padding.1 as i32)));
        }
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        let hp = INNER_PADDING_H + INNER_PADDING_H;
        let vp = INNER_PADDING_V;
        let (cw, ch, _) = if let Some(ref mut child) = self.child {
            child.measure(cmp::max(0, parent_width as i32 - hp) as u16, cmp::max(0, parent_height as i32 - vp) as u16)
        } else {
            (0, 0, false)
        };
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => {
                        let w = cmp::max(cw as i32, self.label_padding.0);
                        cmp::max(0, w as i32 + hp) as u16
                    }
                };
                let h = match control.layout.height {
                    layout::Size::Exact(h) => h,
                    layout::Size::MatchParent => parent_height,
                    layout::Size::WrapContent => {
                        let h = ch as i32 + self.label_padding.1;
                        cmp::max(0, h as i32 + vp) as u16
                    }
                };
                (w, h)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

impl Spawnable for CocoaFrame {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_label("").into_control()
    }
}
extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = common::member_from_cocoa_id_mut::<Frame>(this).unwrap();
        let b2 = common::member_from_cocoa_id_mut::<Frame>(this).unwrap();
        (b.inner().inner().inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: controls::Frame>(this: &mut Frame, _: Sel, param: NSSize) {
    unsafe {
        let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        this.call_on_size::<O>(param.width as u16, param.height as u16);
    }
}
