use super::common::*;

pub use std::os::raw::c_char;

const INNER_PADDING_H: i32 = 7; // TODO: WHY???
const INNER_PADDING_V: i32 = 8; // TODO: WHY???

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        common::register_window_class("PlyguiFrame", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const BASE_CLASS: &str = "NSBox";

pub type Frame = Member<Control<SingleContainer<CocoaFrame>>>;

#[repr(C)]
pub struct CocoaFrame {
    base: common::CocoaControlBase<Frame>,
    label_padding: (i32, i32),
    child: Option<Box<controls::Control>>,
}

impl FrameInner for CocoaFrame {
    fn with_label(label: &str) -> Box<Frame> {
        use plygui_api::controls::HasLabel;

        let mut frame = Box::new(Member::with_inner(
            Control::with_inner(
                SingleContainer::with_inner(
                    CocoaFrame {
                        base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                        label_padding: (0, 0),
                        child: None,
                    },
                    (),
                ),
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = frame.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *frame.as_inner_mut().as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr);
        }
        frame.set_label(label);
        frame
    }
}

impl CocoaFrame {
    fn measure_label(&mut self) {
        let label_size = unsafe { common::measure_nsstring(msg_send![self.base.control, title]) };
        self.label_padding = (label_size.0 as i32, label_size.1 as i32);
    }
}

impl SingleContainerInner for CocoaFrame {
    fn set_child(&mut self, _: &mut MemberBase, child: Option<Box<controls::Control>>) -> Option<Box<controls::Control>> {
        let mut old = self.child.take();
        self.child = child;
        if let Some(ref mut child) = self.child {
            unsafe {
                let child_id = child.native_id() as cocoa_id;
                (&mut *child_id).set_ivar(common::IVAR_PARENT, self.base.control as *mut c_void);
                let () = msg_send![self.base.control, addSubview: child_id];
	            let frame2 = common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap();
	            let (pw, ph) = self.base.measured_size;
	            child.on_added_to_container(
	                frame2,
	                0,
	                INNER_PADDING_V + self.label_padding.1 as i32,
	                cmp::max(0, pw as i32 - INNER_PADDING_H - INNER_PADDING_H) as u16,
	                cmp::max(0, ph as i32 - INNER_PADDING_V - INNER_PADDING_V) as u16,
	            );
            }
        }
        if let Some(ref mut old) = old {
            unsafe {
                let child_id = old.native_id() as cocoa_id;
                *(&mut *child_id).get_mut_ivar::<*mut c_void>(common::IVAR_PARENT) = ptr::null_mut();
                let () = msg_send![child_id, removeFromSuperview];
                let frame2 = common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap();
		        old.on_removed_from_container(frame2);
		    }
        }
        old
    }
    fn child(&self) -> Option<&controls::Control> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut controls::Control> {
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
}

impl ContainerInner for CocoaFrame {
    fn find_control_by_id_mut(&mut self, id: ids::Id) -> Option<&mut controls::Control> {
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id);
            }
        }
        None
    }
    fn find_control_by_id(&self, id: ids::Id) -> Option<&controls::Control> {
        if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id);
            }
        }
        None
    }
}

impl HasLabelInner for CocoaFrame {
    fn label(&self) -> Cow<str> {
        unsafe {
            let label: cocoa_id = msg_send![self.base.control, getTitle];
            let label: *const c_void = msg_send![label, UTF8String];
            ffi::CStr::from_ptr(label as *const c_char).to_string_lossy()
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: &str) {
        unsafe {
            let title = NSString::alloc(cocoa::base::nil).init_str(label);
            let () = msg_send![self.base.control, setTitle: title];
            let () = msg_send![title, release];
        }
        self.measure_label();
    }
}

impl ControlInner for CocoaFrame {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);

        if let Some(ref mut child) = self.child {
            let frame2 = unsafe { common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap() };
            unsafe {
                let () = msg_send![frame2.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview:child.native_id() as cocoa_id];
            }
            let (pw, ph) = self.base.measured_size;
            child.on_added_to_container(
                frame2,
                0,
                INNER_PADDING_V + self.label_padding.1 as i32,
                cmp::max(0, pw as i32 - INNER_PADDING_H - INNER_PADDING_H) as u16,
                cmp::max(0, ph as i32 - INNER_PADDING_V - INNER_PADDING_V) as u16,
            );
        }
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &controls::Container) {
        let frame2 = unsafe { common::member_from_cocoa_id_mut::<Frame>(self.base.control).unwrap() };
        if let Some(ref mut child) = self.child {
            child.on_removed_from_container(frame2);
        }
        unsafe {
            self.base.on_removed_from_container();
        }
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
        use plygui_api::markup::MEMBER_TYPE_FRAME;

        fill_from_markup_base!(self, base, markup, registry, Frame, [MEMBER_TYPE_FRAME]);
        fill_from_markup_label!(self, &mut base.member, markup);
        fill_from_markup_child!(self, &mut base.member, markup, registry);
    }
}

impl MemberInner for CocoaFrame {
    type Id = common::CocoaId;

    fn size(&self) -> (u16, u16) {
        self.base.size()
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        self.base.on_set_visibility(base);
    }

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasLayoutInner for CocoaFrame {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for CocoaFrame {
    fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.base.coords = coords;
        }
        if let Some((x, y)) = self.base.coords {
            let (_, ph) = self.parent().unwrap().size();
            unsafe {
                let mut frame: NSRect = self.base.frame();
                frame.size = NSSize::new((self.base.measured_size.0 as i32) as f64, (self.base.measured_size.1 as i32) as f64);
                frame.origin = NSPoint::new((x) as f64, (ph as i32 - y - self.base.measured_size.1 as i32) as f64);
                let () = msg_send![self.base.control, setFrame: frame];
            }
            if let Some(ref mut child) = self.child {
                child.draw(Some((0, INNER_PADDING_V + self.label_padding.1 as i32)));
            }
        }
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let old_size = self.base.measured_size;
        let hp = INNER_PADDING_H + INNER_PADDING_H + 1;
        let vp = INNER_PADDING_V;
        self.base.measured_size = match member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut measured = false;
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => {
                        let mut w = 0;
                        if let Some(ref mut child) = self.child {
                            let (cw, _, _) = child.measure(max(0, parent_width as i32 - hp) as u16, max(0, parent_height as i32 - vp) as u16);
                            w += max(cw as i32, self.label_padding.0);
                            measured = true;
                        }
                        max(0, w as i32 + hp) as u16
                    }
                };
                let h = match control.layout.height {
                    layout::Size::Exact(h) => h,
                    layout::Size::MatchParent => parent_height,
                    layout::Size::WrapContent => {
                        let mut h = 0;
                        if let Some(ref mut child) = self.child {
                            let ch = if measured {
                                child.size().1
                            } else {
                                let (_, ch, _) = child.measure(max(0, parent_width as i32 - hp) as u16, max(0, parent_height as i32 - vp) as u16);
                                ch
                            };
                            h += ch as i32 + self.label_padding.1;
                        }
                        max(0, h as i32 + vp) as u16
                    }
                };
                (w, h)
            }
        };
        (self.base.measured_size.0, self.base.measured_size.1, self.base.measured_size != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Frame::with_label("").into_control()
}
extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<Frame>(this).unwrap();
        let () = msg_send![super(sp.as_inner_mut().as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        sp.call_on_resize(param.width as u16, param.height as u16);
    }
}
impl_all_defaults!(Frame);
