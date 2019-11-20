use crate::common::{self, *};

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiProgressBar", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const BASE_CLASS: &str = "NSProgressIndicator";

pub type ProgressBar = Member<Control<CocoaProgressBar>>;

#[repr(C)]
pub struct CocoaProgressBar {
    base: common::CocoaControlBase<ProgressBar>,

    skip_callbacks: bool,
}

impl ProgressBarInner for CocoaProgressBar {
    fn with_progress(arg: types::Progress) -> Box<ProgressBar> {
        use plygui_api::controls::HasProgress;

        let mut b = Box::new(Member::with_inner(
            Control::with_inner(
                CocoaProgressBar {
                    base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                    skip_callbacks: false,
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = b.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *b.as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr);
            let () = msg_send![b.as_inner_mut().as_inner_mut().base.control, setMinValue: 0.0];
        }
        b.set_progress(arg);
        b
    }
}

impl HasProgressInner for CocoaProgressBar {
    fn progress(&self, _: &MemberBase) -> types::Progress {
        let total: f64 = unsafe { msg_send![self.base.control, maxValue] };
                
        if total > 0.0 {
            let curr: f64 = unsafe { msg_send![self.base.control, doubleValue] };
            types::Progress::Value(curr as u32, total as u32)
        } else if unsafe { 
            let indeterminated: BOOL = msg_send![self.base.control, isIndeterminate];
            indeterminated == YES
        } {
            types::Progress::Undefined
        } else {
            types::Progress::None
        }
    }
    fn set_progress(&mut self, _: &mut MemberBase, arg: types::Progress) {
        match arg {
            types::Progress::Value(current, total) => unsafe {
                let () = msg_send![self.base.control, setIndeterminate: NO];
                let () = msg_send![self.base.control, setDisplayedWhenStopped: YES];
                let () = msg_send![self.base.control, setMaxValue: if total > 0 { total as f64 } else { 1.0 }];
                let () = msg_send![self.base.control, setDoubleValue: current as f64];
            },
            types::Progress::Undefined => unsafe {
                let () = msg_send![self.base.control, setMaxValue: 0.0];
                let () = msg_send![self.base.control, setDoubleValue: 0.0];
                let () = msg_send![self.base.control, setDisplayedWhenStopped: YES];
                let () = msg_send![self.base.control, setIndeterminate: YES];
            },
            types::Progress::None => unsafe {
                let () = msg_send![self.base.control, setMaxValue: 0.0];
                let () = msg_send![self.base.control, setDoubleValue: 0.0];
                let () = msg_send![self.base.control, setIndeterminate: NO];
                let () = msg_send![self.base.control, setDisplayedWhenStopped: NO];
            },
        }
    }
}

impl ControlInner for CocoaProgressBar {
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
        use plygui_api::markup::MEMBER_TYPE_PROGRESS_BAR;

        fill_from_markup_base!(self, base, markup, registry, ProgressBar, [MEMBER_TYPE_PROGRESS_BAR]);
        fill_from_markup_label!(self, base, markup);
        fill_from_markup_callbacks!(self, markup, registry, [on_click => plygui_api::callbacks::OnClick]);
    }
}

impl HasNativeIdInner for CocoaProgressBar {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaProgressBar {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaProgressBar {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaProgressBar {}

impl HasLayoutInner for CocoaProgressBar {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for CocoaProgressBar {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING as i32 + DEFAULT_PADDING + DEFAULT_PADDING
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
    ProgressBar::with_progress(types::Progress::None).into_control()
}

extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<ProgressBar>(this).unwrap();
        let () = msg_send![super(sp.as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        sp.call_on_size(param.width as u16, param.height as u16)
    }
}
default_impls_as!(ProgressBar);
