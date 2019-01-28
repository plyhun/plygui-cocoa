use super::common::*;

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiSplitted", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
    static ref DELEGATE: common::RefClass = unsafe { register_delegate() };
}
pub type Splitted = Member<Control<MultiContainer<CocoaSplitted>>>;

const BASE_CLASS: &str = "NSSplitView";
const PADDING: f64 = 4.0; // TODO WHY??

#[repr(C)]
pub struct CocoaSplitted {
    base: common::CocoaControlBase<Splitted>,
    splitter: f32,
    first: Box<dyn controls::Control>,
    second: Box<dyn controls::Control>,
}

impl CocoaSplitted {
    fn children_sizes(&self) -> (u16, u16) {
        let (w, h) = self.size();
        let splitter: f32 = unsafe { msg_send![self.base.control, dividerThickness] };
        let target = match self.layout_orientation() {
            layout::Orientation::Horizontal => w,
            layout::Orientation::Vertical => h,
        };
        (
            utils::coord_to_size((target as f32 * self.splitter) as i32 - (splitter as i32) - PADDING as i32),
            utils::coord_to_size((target as f32 * (1.0 - self.splitter)) as i32 - (splitter as i32) - PADDING as i32),
        )
    }
    fn update_splitter(&mut self) {
        let orientation = self.layout_orientation();
        let () = match orientation {
            layout::Orientation::Horizontal => unsafe { msg_send![self.base.control, setPosition:(self.base.measured_size.0 as f32 * self.splitter) ofDividerAtIndex:0] },
            layout::Orientation::Vertical => unsafe { msg_send![self.base.control, setPosition:(self.base.measured_size.1 as f32 * self.splitter) ofDividerAtIndex:0] },
        };
    }
}

impl SplittedInner for CocoaSplitted {
    fn with_content(first: Box<dyn controls::Control>, second: Box<dyn controls::Control>, orientation: layout::Orientation) -> Box<Splitted> {
        let mut ll = Box::new(Member::with_inner(
            Control::with_inner(
                MultiContainer::with_inner(
                    CocoaSplitted {
                        base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                        splitter: defaults::SPLITTED_POSITION,
                        first: first,
                        second: second,
                    },
                    (),
                ),
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = ll.as_mut() as *mut _ as *mut ::std::os::raw::c_void;

        unsafe {
            (&mut *ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr);
            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(common::IVAR, selfptr);
            let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, setDelegate: delegate];
            let first = ll.as_inner_mut().as_inner_mut().as_inner_mut().first.native_id() as cocoa_id;
            let second = ll.as_inner_mut().as_inner_mut().as_inner_mut().second.native_id() as cocoa_id;
            let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview: first];
            let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview: second];
            let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, setVertical: orientation_to_vertical(orientation)];
            let () = msg_send![ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control, adjustSubviews];
        }
        ll
    }
    fn set_splitter(&mut self, _: &mut MemberBase, _: &mut ControlBase, pos: f32) {
        let pos = pos % 1.0;
        self.splitter = pos;
        self.update_splitter();
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
    fn find_control_by_id_mut(&mut self, id_: ids::Id) -> Option<&mut dyn controls::Control> {
        if self.first().as_member().id() == id_ {
            return Some(self.first_mut());
        }
        if self.second().as_member().id() == id_ {
            return Some(self.second_mut());
        }

        let self2: &mut Self = unsafe { mem::transmute(self as *mut Self) }; // bck is stupid
        if let Some(c) = self.first_mut().is_container_mut() {
            let ret = c.find_control_by_id_mut(id_);
            if ret.is_some() {
                return ret;
            }
        }
        if let Some(c) = self2.second_mut().is_container_mut() {
            let ret = c.find_control_by_id_mut(id_);
            if ret.is_some() {
                return ret;
            }
        }

        None
    }
    fn find_control_by_id(&self, id_: ids::Id) -> Option<&dyn controls::Control> {
        if self.first().as_member().id() == id_ {
            return Some(self.first());
        }
        if self.second().as_member().id() == id_ {
            return Some(self.second());
        }

        if let Some(c) = self.first().is_container() {
            let ret = c.find_control_by_id(id_);
            if ret.is_some() {
                return ret;
            }
        }
        if let Some(c) = self.second().is_container() {
            let ret = c.find_control_by_id(id_);
            if ret.is_some() {
                return ret;
            }
        }

        None
    }
}

impl HasOrientationInner for CocoaSplitted {
    fn layout_orientation(&self) -> layout::Orientation {
        vertical_to_orientation(unsafe { msg_send![self.base.control, isVertical] })
    }
    fn set_layout_orientation(&mut self, _: &mut MemberBase, orientation: layout::Orientation) {
        if orientation != self.layout_orientation() {
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
        let self2: &mut Splitted = unsafe { utils::base_to_impl_mut(member) };
        let control = self.base.control;
        let (first_size, second_size) = self.children_sizes();
        match self.layout_orientation() {
            layout::Orientation::Horizontal => {
                let h = utils::coord_to_size(ph as i32);
                unsafe {
                    let () = msg_send![control, addSubview: self.first.native_id() as cocoa_id];
                }
                self.first.on_added_to_container(self2, 0, 0, first_size, h);
                unsafe {
                    let () = msg_send![control, addSubview: self.second.native_id() as cocoa_id];
                }
                self.second.on_added_to_container(self2, first_size as i32, 0, second_size, h);
            }
            layout::Orientation::Vertical => {
                let w = utils::coord_to_size(pw as i32);
                unsafe {
                    let () = msg_send![control, addSubview: self.first.native_id() as cocoa_id];
                }
                self.first.on_added_to_container(self2, 0, 0, w, first_size);
                unsafe {
                    let () = msg_send![control, addSubview: self.second.native_id() as cocoa_id];
                }
                self.second.on_added_to_container(self2, 0, first_size as i32, w, second_size);
            }
        }
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
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.update_splitter();
        self.base.invalidate();
    }
}

impl MemberInner for CocoaSplitted {
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

impl Drawable for CocoaSplitted {
    fn draw(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, coords: Option<(i32, i32)>) {
        self.base.draw(coords);
        let splitter: f32 = unsafe {msg_send![self.base.control, dividerThickness]};
        let o = self.layout_orientation();
        let (first, _) = self.children_sizes();
        let (pw, ph) = self.base.measured_size;
        let (fw, fh) = self.first.size();
        let (sw, sh) = self.second.size();
        // TODO why children of splitted are drawn from top rather from bottom?
        match o {
        	layout::Orientation::Horizontal => {
	        	self.first.draw(Some((0, ph as i32 - fh as i32)));
                self.second.draw(Some((first as i32 + splitter as i32 + PADDING as i32 + PADDING as i32, ph as i32 - sh as i32)));
        	},
        	layout::Orientation::Vertical => {
	        	self.first.draw(Some((pw as i32 - fw as i32, 0)));
                self.second.draw(Some((pw as i32 - sw as i32, first as i32 + splitter as i32 + PADDING as i32 + PADDING as i32)));
        	},
        }
    }
    fn measure(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        let orientation = self.layout_orientation();
        let old_size = self.base.measured_size;
        let (first_size, second_size) = self.children_sizes();
        self.base.measured_size = match member.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut measured = false;
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => {
                        let mut w = 0;
                        for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
                            match orientation {
                                layout::Orientation::Horizontal => {
                                    let (cw, _, _) = child.measure(cmp::max(0, *size) as u16, cmp::max(0, parent_height as i32) as u16);
                                    w += cw;
                                }
                                layout::Orientation::Vertical => {
                                    let (cw, _, _) = child.measure(cmp::max(0, parent_width as i32) as u16, cmp::max(0, *size) as u16);
                                    w = cmp::max(w, cw);
                                }
                            }
                        }
                        measured = true;
                        cmp::max(0, w as i32) as u16
                    }
                };
                let h = match control.layout.height {
                    layout::Size::Exact(h) => h,
                    layout::Size::MatchParent => parent_height,
                    layout::Size::WrapContent => {
                        let mut h = 0;
                        for (size, child) in [(first_size, self.first.as_mut()), (second_size, self.second.as_mut())].iter_mut() {
                            let ch = if measured {
                                child.size().1
                            } else {
                                let (_, ch, _) = match orientation {
                                    layout::Orientation::Horizontal => child.measure(cmp::max(0, *size) as u16, cmp::max(0, parent_height as i32) as u16),
                                    layout::Orientation::Vertical => child.measure(cmp::max(0, parent_width as i32) as u16, cmp::max(0, *size) as u16),
                                };
                                ch
                            };
                            match orientation {
                                layout::Orientation::Horizontal => {
                                    h = cmp::max(h, ch);
                                }
                                layout::Orientation::Vertical => {
                                    h += ch;
                                }
                            }
                        }
                        cmp::max(0, h as i32) as u16
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

#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Splitted::with_content(text::Text::empty().into_control(), text::Text::empty().into_control(), layout::Orientation::Horizontal).into_control()
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
    sp.as_inner_mut().as_inner_mut().as_inner_mut().base.invalidate();
}
extern "C" fn splitter_moved(this: &mut Object, _: Sel, _: cocoa_id) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<Splitted>(this).unwrap();
        let subviews: cocoa_id = msg_send![sp.as_inner_mut().as_inner_mut().as_inner_mut().base.control, subviews];
        let first: cocoa_id = msg_send![subviews, objectAtIndex:0];
        let first: NSRect = msg_send![first, frame];
        let second: cocoa_id = msg_send![subviews, objectAtIndex:1];
        let second: NSRect = msg_send![second, frame];
        let size: NSRect = msg_send![sp.as_inner_mut().as_inner_mut().as_inner_mut().base.control, frame];
        let o = sp.as_inner().as_inner().as_inner().layout_orientation();
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
        let old_splitter = sp.as_inner_mut().as_inner_mut().as_inner_mut().splitter;
        let member = &mut *(sp.base_mut() as *mut MemberBase);
        let control = &mut *(sp.as_inner_mut().base_mut() as *mut ControlBase);
        if (old_splitter - splitter_first) != 0.0 {
            let sp = sp.as_inner_mut().as_inner_mut().as_inner_mut();
            sp.splitter = splitter_first;
            sp.measure(member, control, size.size.width as u16, size.size.height as u16);
            sp.draw(member, control, None);
        }
    }
}
extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<Splitted>(this).unwrap();
        let () = msg_send![super(sp.as_inner_mut().as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        sp.call_on_resize(param.width as u16, param.height as u16)
    }
}
impl_all_defaults!(Splitted);
