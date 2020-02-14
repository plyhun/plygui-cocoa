use crate::common::{self, *};

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiSplitted", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
    static ref DELEGATE: common::RefClass = unsafe { register_delegate() };
}
pub type Splitted = AMember<AControl<AContainer<AMultiContainer<ASplitted<CocoaSplitted>>>>>;

const BASE_CLASS: &str = "NSSplitView";
const PADDING: i32 = 4; // TODO WHY??

#[repr(C)]
pub struct CocoaSplitted {
    base: common::CocoaControlBase<Splitted>,
    splitter: f32,
    first: Box<dyn controls::Control>,
    second: Box<dyn controls::Control>,
}

impl CocoaSplitted {
    fn children_sizes(&self, member: &MemberBase, control: &ControlBase) -> (u16, u16) {
        let (w, h) = self.base.size(control);
        let splitter: f32 = unsafe { msg_send![self.base.control, dividerThickness] };
        let target = match self.orientation(member) {
            layout::Orientation::Horizontal => w,
            layout::Orientation::Vertical => h,
        };
        (
            utils::coord_to_size((target as f32 * self.splitter) as i32 - (splitter as i32) - PADDING),
            utils::coord_to_size((target as f32 * (1.0 - self.splitter)) as i32 - (splitter as i32) - PADDING),
        )
    }
    fn update_splitter(&mut self, member: &MemberBase, control: &ControlBase) {
        let orientation = self.orientation(member);
        let () = match orientation {
            layout::Orientation::Horizontal => unsafe { msg_send![self.base.control, setPosition:(control.measured.0 as f32 * self.splitter) ofDividerAtIndex:0] },
            layout::Orientation::Vertical => unsafe { msg_send![self.base.control, setPosition:(control.measured.1 as f32 * self.splitter) ofDividerAtIndex:0] },
        };
    }
    fn draw_children(&mut self, member: &MemberBase, control: &ControlBase) {
        let splitter: f32 = unsafe { msg_send![self.base.control, dividerThickness] };
        let o = self.orientation(member);
        let (first, _) = self.children_sizes(member, control);
        let (pw, ph) = control.measured;
        let (fw, fh) = self.first.size();
        let (sw, sh) = self.second.size();
        // TODO why children of splitted are drawn from top rather from bottom?
        match o {
            layout::Orientation::Horizontal => {
                self.first.draw(Some((0, ph as i32 - fh as i32)));
                self.second.draw(Some((first as i32 + splitter as i32 + PADDING + PADDING, ph as i32 - sh as i32)));
            }
            layout::Orientation::Vertical => {
                self.first.draw(Some((pw as i32 - fw as i32, 0)));
                self.second.draw(Some((pw as i32 - sw as i32, first as i32 + splitter as i32 + PADDING + PADDING)));
            }
        }
    }
    fn update_children_layout(&mut self, member: &MemberBase, control: &ControlBase) -> (u16, u16) {
        let orientation = self.orientation(member);
        let (first_size, second_size) = self.children_sizes(member, control);
        let (width, height) = control.measured;
        let mut w = 0;
        let mut h = 0;
        for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
            match orientation {
                layout::Orientation::Horizontal => {
                    let (cw, ch, _) = child.measure(cmp::max(0, *size) as u16, cmp::max(0, height as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16);
                    w += cw;
                    h = cmp::max(h, ch);
                }
                layout::Orientation::Vertical => {
                    let (cw, ch, _) = child.measure(cmp::max(0, width as i32 - DEFAULT_PADDING - DEFAULT_PADDING) as u16, cmp::max(0, *size) as u16);
                    w = cmp::max(w, cw);
                    h += ch;
                }
            }
        }
        (w, h)
    }
}

impl<O: controls::Splitted> NewSplittedInner<O> for CocoaSplitted {
    fn with_uninit_params(ptr: &mut mem::MaybeUninit<O>, first: Box<dyn controls::Control>, second: Box<dyn controls::Control>, orientation: layout::Orientation) -> Self {
        let sp = CocoaSplitted {
            base: common::CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>),
            splitter: defaults::SPLITTED_POSITION,
            first: first,
            second: second,
        };
        unsafe {
            let selfptr = ptr as *mut _ as *mut Splitted;
            (&mut *sp.base.control).set_ivar(common::IVAR, selfptr as *mut c_void);
            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(common::IVAR, selfptr as *mut c_void);
            let () = msg_send![sp.base.control, setDelegate: delegate];
            let first = sp.first.native_id() as cocoa_id;
            let second = sp.second.native_id() as cocoa_id;
            let () = msg_send![sp.base.control, addSubview: first];
            let () = msg_send![sp.base.control, addSubview: second];
            let () = msg_send![sp.base.control, setVertical: orientation_to_vertical(orientation)];
            let () = msg_send![sp.base.control, adjustSubviews];
        }
        sp
    }
}
impl SplittedInner for CocoaSplitted {
    fn with_content(first: Box<dyn controls::Control>, second: Box<dyn controls::Control>, orientation: layout::Orientation) -> Box<dyn controls::Splitted> {
        let mut b: Box<mem::MaybeUninit<Splitted>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AMultiContainer::with_inner(
                        ASplitted::with_inner(
                            <Self as NewSplittedInner<Splitted>>::with_uninit_params(b.as_mut(), first, second, orientation)
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
    fn set_splitter(&mut self, base: &mut MemberBase, pos: f32) {
        let (m, c, _) = unsafe { Splitted::control_base_parts_mut(base) };
        let pos = pos % 1.0;
        self.splitter = pos;
        self.update_splitter(m, c);
        self.base.invalidate();
    }
    fn splitter(&self) -> f32 {
        self.splitter
    }
    fn first(&self) -> &dyn controls::Control {
        self.first.as_ref()
    }
    fn second(&self) -> &dyn controls::Control {
        self.second.as_ref()
    }
    fn first_mut(&mut self) -> &mut dyn controls::Control {
        self.first.as_mut()
    }
    fn second_mut(&mut self) -> &mut dyn controls::Control {
        self.second.as_mut()
    }
}
impl MultiContainerInner for CocoaSplitted {
    fn len(&self) -> usize {
        2
    }
    fn set_child_to(&mut self, base: &mut MemberBase, index: usize, mut child: Box<dyn controls::Control>) -> Option<Box<dyn controls::Control>> {
        match index {
            0 => unsafe {
                let self2 = utils::base_to_impl_mut::<Splitted>(base);
                let sizes = self.first.size();
                let () = msg_send![self.first.native_id() as cocoa_id, removeFromSuperview];
                if self.base.root().is_some() {
                    self.first.on_removed_from_container(self2);
                }
                let () = msg_send![child.native_id() as cocoa_id, addSubview: child.native_id() as cocoa_id];
                if self.base.root().is_some() {
                    child.on_added_to_container(self2, 0, 0, sizes.0, sizes.1);
                }
                mem::swap(&mut self.first, &mut child);
            },
            1 => unsafe {
                let self2 = utils::base_to_impl_mut::<Splitted>(base);
                let sizes = self.second.size();
                let () = msg_send![self.second.native_id() as cocoa_id, removeFromSuperview];
                if self.base.root().is_some() {
                    self.second.on_removed_from_container(self2);
                }
                let () = msg_send![child.native_id() as cocoa_id, addSubview: child.native_id() as cocoa_id];
                if self.base.root().is_some() {
                    child.on_added_to_container(self2, 0, 0, sizes.0, sizes.1);
                }
                mem::swap(&mut self.second, &mut child);
            },
            _ => return None,
        }
        self.base.invalidate();
        Some(child)
    }
    fn remove_child_from(&mut self, _: &mut MemberBase, _: usize) -> Option<Box<dyn controls::Control>> {
        None
    }
    fn child_at(&self, index: usize) -> Option<&dyn controls::Control> {
        match index {
            0 => Some(self.first()),
            1 => Some(self.second()),
            _ => None,
        }
    }
    fn child_at_mut(&mut self, index: usize) -> Option<&mut dyn controls::Control> {
        match index {
            0 => Some(self.first_mut()),
            1 => Some(self.second_mut()),
            _ => None,
        }
    }
}

impl ContainerInner for CocoaSplitted {
    fn find_control_mut(&mut self, arg: types::FindBy) -> Option<&mut dyn controls::Control> {
        match arg {
            types::FindBy::Id(id) => {
                if self.first().as_member().id() == id {
                    return Some(self.first_mut());
                }
                if self.second().as_member().id() == id {
                    return Some(self.second_mut());
                }
            }
            types::FindBy::Tag(ref tag) => {
                if let Some(mytag) = self.first.as_member().tag() {
                    if tag.as_str() == mytag {
                        return Some(self.first_mut());
                    }
                }
                if let Some(mytag) = self.second.as_member().tag() {
                    if tag.as_str() == mytag {
                        return Some(self.second_mut());
                    }
                }
            }
        }

        let self2: &mut CocoaSplitted = unsafe { mem::transmute(self as *mut CocoaSplitted) }; // bck is stupid
        if let Some(c) = self.first_mut().is_container_mut() {
            let ret = c.find_control_mut(arg.clone());
            if ret.is_some() {
                return ret;
            }
        }
        if let Some(c) = self2.second_mut().is_container_mut() {
            let ret = c.find_control_mut(arg);
            if ret.is_some() {
                return ret;
            }
        }
        None
    }
    fn find_control(&self, arg: types::FindBy) -> Option<&dyn controls::Control> {
        match arg {
            types::FindBy::Id(id) => {
                if self.first().as_member().id() == id {
                    return Some(self.first());
                }
                if self.second().as_member().id() == id {
                    return Some(self.second());
                }
            }
            types::FindBy::Tag(ref tag) => {
                if let Some(mytag) = self.first.as_member().tag() {
                    if tag.as_str() == mytag {
                        return Some(self.first.as_ref());
                    }
                }
                if let Some(mytag) = self.second.as_member().tag() {
                    if tag.as_str() == mytag {
                        return Some(self.second.as_ref());
                    }
                }
            }
        }
        if let Some(c) = self.first().is_container() {
            let ret = c.find_control(arg.clone());
            if ret.is_some() {
                return ret;
            }
        }
        if let Some(c) = self.second().is_container() {
            let ret = c.find_control(arg);
            if ret.is_some() {
                return ret;
            }
        }
        None
    }
}

impl HasOrientationInner for CocoaSplitted {
    fn orientation(&self, _: &MemberBase) -> layout::Orientation {
        vertical_to_orientation(unsafe { msg_send![self.base.control, isVertical] })
    }
    fn set_orientation(&mut self, base: &mut MemberBase, orientation: layout::Orientation) {
        if orientation != self.orientation(base) {
            unsafe {
                let () = msg_send![self.base.control, setVertical: orientation_to_vertical(orientation)];
            }
            self.base.invalidate();
        }
    }
}

impl ControlInner for CocoaSplitted {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, _: i32, _: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        let (first_size, second_size) = self.children_sizes(member, control);
        match self.orientation(member) {
            layout::Orientation::Horizontal => {
                let h = utils::coord_to_size(ph as i32);
                let self2: &mut Splitted = unsafe { utils::base_to_impl_mut(member) };
                self.first.on_added_to_container(self2, 0, 0, first_size, h);
                self.second.on_added_to_container(self2, first_size as i32, 0, second_size, h);
            }
            layout::Orientation::Vertical => {
                let w = utils::coord_to_size(pw as i32);
                let self2: &mut Splitted = unsafe { utils::base_to_impl_mut(member) };
                self.first.on_added_to_container(self2, 0, 0, w, first_size);
                self.second.on_added_to_container(self2, 0, first_size as i32, w, second_size);
            }
        }
        self.update_children_layout(member, control);
        self.draw_children(member, control);
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        let self2: &Splitted = unsafe { common::member_from_cocoa_id(self.base.control).unwrap() };
        self.first.on_removed_from_container(self2);
        self.second.on_removed_from_container(self2);
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
        use plygui_api::markup::MEMBER_TYPE_SPLITTED;

        fill_from_markup_base!(self, base, markup, registry, Splitted, [MEMBER_TYPE_SPLITTED]);
        fill_from_markup_children!(self, base, markup, registry);
    }
}

impl HasLayoutInner for CocoaSplitted {
    fn on_layout_changed(&mut self, base: &mut MemberBase) {
        let (m, c, _) = unsafe { Splitted::control_base_parts_mut(base) };
        self.update_splitter(m, c);
        self.base.invalidate();
    }
}

impl HasNativeIdInner for CocoaSplitted {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaSplitted {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaSplitted {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaSplitted {}

impl Drawable for CocoaSplitted {
    fn draw(&mut self, member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
        self.draw_children(member, control);
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let (w, h) = self.update_children_layout(member, control);
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => cmp::max(0, w as i32) as u16
                };
                let h = match control.layout.height {
                    layout::Size::Exact(h) => h,
                    layout::Size::MatchParent => parent_height,
                    layout::Size::WrapContent => cmp::max(0, h as i32) as u16
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
fn orientation_to_vertical(orientation: layout::Orientation) -> BOOL {
    match orientation {
        layout::Orientation::Horizontal => YES,
        layout::Orientation::Vertical => NO,
    }
}
fn vertical_to_orientation(vertical: BOOL) -> layout::Orientation {
    match vertical {
        YES => layout::Orientation::Horizontal,
        NO => layout::Orientation::Vertical,
        _ => unreachable!(),
    }
}
impl Spawnable for CocoaSplitted {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_content(super::text::Text::spawn(), super::text::Text::spawn(), layout::Orientation::Vertical).into_control()
    }
}
unsafe fn register_delegate() -> common::RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiSplitterDelegate", superclass).unwrap();

    decl.add_method(sel!(splitViewDidResizeSubviews:), splitter_moved as extern "C" fn(&mut Object, Sel, cocoa_id));
    decl.add_method(sel!(splitView:resizeSubviewsWithOldSize:), splitter_resize_subviews as extern "C" fn(&mut Object, Sel, NSSize, cocoa_id));
    decl.add_method(sel!(shouldAdjustSizeOfSubview:), adjust_subview_size as extern "C" fn(&mut Object, Sel, cocoa_id) -> BOOL);
    decl.add_ivar::<*mut c_void>(common::IVAR);

    common::RefClass(decl.register())
}
extern "C" fn adjust_subview_size(_: &mut Object, _: Sel, _: cocoa_id) -> BOOL {
    NO
}
extern "C" fn splitter_resize_subviews(this: &mut Object, _: Sel, _: NSSize, _: cocoa_id) {
    let sp = unsafe { common::member_from_cocoa_id_mut::<Splitted>(this).unwrap() };
    let sp2 = unsafe { common::member_from_cocoa_id_mut::<Splitted>(this).unwrap() };
    let (m, c, _) = sp2.as_control_parts_mut();
    sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().update_children_layout(m, c);
    OuterDrawable::draw(sp, None);
}
extern "C" fn splitter_moved(this: &mut Object, _: Sel, _: cocoa_id) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<Splitted>(this).unwrap();
        let subviews: cocoa_id = msg_send![sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, subviews];
        let first: cocoa_id = msg_send![subviews, objectAtIndex:0];
        let first: NSRect = msg_send![first, frame];
        let second: cocoa_id = msg_send![subviews, objectAtIndex:1];
        let second: NSRect = msg_send![second, frame];
        let size: NSRect = msg_send![sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, frame];
        let o = controls::HasOrientation::orientation(sp);
        let mut splitter_first = match o {
            layout::Orientation::Horizontal => (first.size.width / size.size.width),
            layout::Orientation::Vertical => (first.size.height / size.size.height),
        } as f32;
        let splitter_second = match o {
            layout::Orientation::Horizontal => (second.size.width / size.size.width),
            layout::Orientation::Vertical => (second.size.height / size.size.height),
        } as f32;
        if splitter_first.is_nan() || splitter_second.is_nan() {
            return;
        }
        let bias = (1.0 - (splitter_first + splitter_second)) / 2.0;
        splitter_first += bias;
        let old_splitter = sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().splitter;
        if (old_splitter - splitter_first) != 0.0 {
            sp.set_skip_draw(true);
            {
                let base = common::member_base_from_cocoa_id_mut(this).unwrap();
                let (m, c, _) = Splitted::control_base_parts_mut(base);
                let sp = sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
                // On first appearance NSSplitView loads its own default divider position, ignoring the sizing of the second child.
                // We can use it to distinguish the initial appearance from all the following, which have to control our splitter position.
                if second.size.width >= 1.0 && second.size.height >= 1.0 {
                    sp.splitter = splitter_first;
                } 
                sp.update_children_layout(m, c);
                sp.draw_children(m, c);
            }
            sp.set_skip_draw(false);
        }
    }
}
extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = common::member_from_cocoa_id_mut::<Splitted>(this).unwrap();
        let b2 = common::member_from_cocoa_id_mut::<Splitted>(this).unwrap();
        (b.inner().inner().inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: controls::Splitted>(this: &mut Splitted, _: Sel, param: NSSize) {
    unsafe {
        let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        this.call_on_size::<O>(param.width as u16, param.height as u16)
    }
}
