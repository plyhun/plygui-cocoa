use crate::common::{self, *};

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiLinearLayout", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const BASE_CLASS: &str = "NSView";

pub type LinearLayout = AMember<AControl<AContainer<AMultiContainer<ALinearLayout<CocoaLinearLayout>>>>>;

#[repr(C)]
pub struct CocoaLinearLayout {
    base: common::CocoaControlBase<LinearLayout>,
    orientation: layout::Orientation,
    children: Vec<Box<dyn controls::Control>>,
}

impl<O: controls::LinearLayout> NewLinearLayoutInner<O> for CocoaLinearLayout {
    fn with_uninit_params(ptr: &mut mem::MaybeUninit<O>, orientation: layout::Orientation) -> Self {
        let ll = CocoaLinearLayout {
            base: common::CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>),
            orientation: orientation,

            children: Vec::new(),
        };
        let selfptr = ptr as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *ll.base.control).set_ivar(common::IVAR, selfptr);
        }
        ll
    }
}
impl LinearLayoutInner for CocoaLinearLayout {
    fn with_orientation(orientation: layout::Orientation) -> Box<dyn controls::LinearLayout> {
        let mut b: Box<mem::MaybeUninit<LinearLayout>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AMultiContainer::with_inner(
                        ALinearLayout::with_inner(
                            <Self as NewLinearLayoutInner<LinearLayout>>::with_uninit_params(b.as_mut(), orientation),
                        )
                    ),
                )
            ),
        );
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
}
impl MultiContainerInner for CocoaLinearLayout {
    fn len(&self) -> usize {
        self.children.len()
    }
    fn set_child_to(&mut self, base: &mut MemberBase, index: usize, new: Box<dyn controls::Control>) -> Option<Box<dyn controls::Control>> {
        let mut old = self.remove_child_from(base, index);

        let this = unsafe { common::member_from_cocoa_id::<LinearLayout>(self.base.control).unwrap() };
        unsafe {
            if let Some(ref mut old) = old {
                if self.base.root().is_some() {
                    old.on_removed_from_container(this);
                }
                let () = msg_send![old.native_id() as cocoa_id, removeFromSuperview];
            }
            let () = msg_send![self.base.control, addSubview: new.native_id() as cocoa_id];
        }
        self.children.insert(index, new);
        let (w, h) = self.base.size(&this.inner().base);
        
        let (cw, ch) = {
            let mut w = 0;
            let mut h = 0;
            for i in 0..index {
                let (cw, ch) = self.children[i].size();
                w += cw;
                h += ch;
            }
            (w as i32, h as i32)
        };
        
        if self.base.root().is_some() {
            match self.orientation {
                layout::Orientation::Vertical => {
                    self.children.get_mut(index).unwrap().on_added_to_container(
                        this, 0, ch, utils::coord_to_size(w as i32), utils::coord_to_size(h as i32 - ch)
                    );
                },
                layout::Orientation::Horizontal => {
                    self.children.get_mut(index).unwrap().on_added_to_container(
                        this, cw, 0, utils::coord_to_size(w as i32 - cw), utils::coord_to_size(h as i32)
                    );
                }
            }
        }
        self.base.invalidate();

        old
    }
    fn remove_child_from(&mut self, _: &mut MemberBase, index: usize) -> Option<Box<dyn controls::Control>> {
        if index >= self.children.len() {
            return None;
        }
        let mut child = self.children.remove(index);
        if self.base.root().is_some() {
            child.on_removed_from_container(unsafe { common::member_from_cocoa_id::<LinearLayout>(self.base.control).unwrap() });
        }
        self.base.invalidate();

        Some(child)
    }
    fn child_at(&self, index: usize) -> Option<&dyn controls::Control> {
        self.children.get(index).map(|c| c.as_ref())
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut dyn controls::Control> {
        if let Some(c) = self.children.get_mut(index) {
            Some(c.as_mut())
        } else {
            None
        }
    }
}

impl ContainerInner for CocoaLinearLayout {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        for child in self.children.as_mut_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if child.as_member_mut().id() == *id {
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
                let ret = c.find_control_mut(arg.clone());
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
    }
    fn find_control<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
        for child in self.children.as_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if child.as_member().id() == *id {
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
                let ret = c.find_control(arg.clone());
                if ret.is_none() {
                    continue;
                }
                return ret;
            }
        }
        None
    }
}

impl HasOrientationInner for CocoaLinearLayout {
    fn orientation(&self, _: &MemberBase) -> layout::Orientation {
        self.orientation
    }
    fn set_orientation(&mut self, _: &mut MemberBase, orientation: layout::Orientation) {
        if orientation != self.orientation {
            self.orientation = orientation;
            self.base.invalidate();
        }
    }
}

impl ControlInner for CocoaLinearLayout {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        let orientation = self.orientation;
        control.coords = Some((x, y));
        let mut x = 0;
        let mut y = 0;
        let mut pw = pw as i32;
        let mut ph = ph as i32;

        let self2 = unsafe { common::member_from_cocoa_id_mut::<LinearLayout>(self.base.control).unwrap() };
        for ref mut child in self.children.as_mut_slice() {
            unsafe {
                let () = msg_send![self2.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, addSubview: child.native_id() as cocoa_id];
            }
            child.on_added_to_container(
                self2, x, y, 
                utils::coord_to_size(pw),
                utils::coord_to_size(ph)
            );
            let (xx, yy) = child.size();
            match orientation {
                layout::Orientation::Horizontal => {
                    x += xx as i32;
                    pw -= xx as i32;
                }
                layout::Orientation::Vertical => {
                    y += yy as i32;
                    ph -= yy as i32;
                }
            }
        }
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        let ll2: &LinearLayout = unsafe { common::member_from_cocoa_id(self.base.control).unwrap() };
        for ref mut child in self.children.as_mut_slice() {
            child.on_removed_from_container(ll2);
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
        use plygui_api::markup::MEMBER_TYPE_LINEAR_LAYOUT;

        fill_from_markup_base!(self, base, markup, registry, LinearLayout, [MEMBER_TYPE_LINEAR_LAYOUT]);
        fill_from_markup_children!(self, base, markup, registry);
    }
}

impl HasLayoutInner for CocoaLinearLayout {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl HasNativeIdInner for CocoaLinearLayout {
    type Id = common::CocoaId;

    fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaLinearLayout {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaLinearLayout {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaLinearLayout {}

impl Drawable for CocoaLinearLayout {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
        let mut x = 0;
        let mut y = 0;

        for child in self.children.as_mut_slice() {
            let child_size = child.size();
            child.draw(Some((x, y)));
            match self.orientation {
                layout::Orientation::Horizontal => x += child_size.0 as i32,
                layout::Orientation::Vertical => y += child_size.1 as i32,
            }
        }
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let orientation = self.orientation;
        let old_size = control.measured;
        let mut w = 0;
        let mut h = 0;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                for child in self.children.as_mut_slice() {
                    match orientation {
                        layout::Orientation::Horizontal => {
                            let (cw, ch, _) = child.measure(max(0, parent_width as i32 - w as i32) as u16, max(0, parent_height as i32) as u16);
                            w += cw;
                            h = max(h, ch);
                        }
                        layout::Orientation::Vertical => {
                            let (cw, ch, _) = child.measure(max(0, parent_width as i32) as u16, max(0, parent_height as i32 - h as i32) as u16);
                            w = max(w, cw);
                            h += ch;
                        }
                    }
                }
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => {
                        max(0, w as i32) as u16
                    }
                };
                let h = match control.layout.height {
                    layout::Size::Exact(h) => h,
                    layout::Size::MatchParent => parent_height,
                    layout::Size::WrapContent => {
                        max(0, h as i32) as u16
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
impl Spawnable for CocoaLinearLayout {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_orientation(layout::Orientation::Vertical).into_control()
    }
}
extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = common::member_from_cocoa_id_mut::<LinearLayout>(this).unwrap();
        let b2 = common::member_from_cocoa_id_mut::<LinearLayout>(this).unwrap();
        (b.inner().inner().inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: controls::LinearLayout>(this: &mut LinearLayout, _: Sel, param: NSSize) {
    unsafe {
        let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        this.call_on_size::<O>(param.width as u16, param.height as u16)
    }
}
