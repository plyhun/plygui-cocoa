use crate::common::{self, *};

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiList", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
            decl.add_method(sel!(numberOfRowsInTableView:), datasource_len as extern "C" fn(&mut Object, Sel, cocoa_id) -> NSInteger);
            //decl.add_method(sel!(tableView:objectValueForTableColumn:row:), get_item as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id, NSInteger) -> cocoa_id);
            decl.add_method(sel!(tableView:viewForTableColumn:row:), spawn_item as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id, NSInteger) -> cocoa_id);
        })
    };
}

const BASE_CLASS: &str = "NSTableView";

pub type List = Member<Control<Adapter<CocoaList>>>;

#[repr(C)]
pub struct CocoaList {
    base: common::CocoaControlBase<List>,
    children: Vec<Box<dyn controls::Control>>,
}

impl AdapterViewInner for CocoaList {
    fn with_adapter(adapter: Box<dyn types::Adapter>) -> Box<List> {
        let mut ll = Box::new(Member::with_inner(
            Control::with_inner(
                Adapter::with_inner(
                    CocoaList {
                        base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                        children: Vec::new(),
                    },
                    adapter,
                ),
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = ll.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            let control = ll.as_inner_mut().as_inner_mut().as_inner_mut().base.control;
            (&mut *control).set_ivar(common::IVAR, selfptr);
            
            let column: cocoa_id = msg_send![Class::get("NSTableColumn").unwrap(), alloc];
            let ident = NSString::alloc(nil).init_str("_");
            let column: cocoa_id = msg_send![column, initWithIdentifier:ident];
            let () = msg_send![control, addTableColumn: column];
            
            let () = msg_send![control, setDelegate: control];
            let () = msg_send![control, setDataSource: control];
        }
        ll
    }
    fn on_item_change(&mut self, base: &mut MemberBase, value: types::Change) {
        let mut y = 0;
        {
            for item in self.children.as_slice() {
                let (_, yy) = item.size();
                y += yy as i32;
            }
        }
        match value {
            types::Change::Added(at) => {
                //self.add_item_inner(base, at, &mut y);
            },
            types::Change::Removed(at) => {
                //self.remove_item_inner(base, at);
            },
            types::Change::Edited(_) => {
            },
        }
        self.base.invalidate();
    }
}

impl ContainerInner for CocoaList {
    fn find_control_mut(&mut self, arg: types::FindBy) -> Option<&mut dyn controls::Control> {
        for child in self.children.as_mut_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if child.as_member_mut().id() == *id {
                        return Some(child.as_mut());
                    }
                }
                types::FindBy::Tag(ref tag) => {
                    if let Some(mytag) = child.as_member_mut().tag() {
                        if tag.as_str() == mytag {
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
    fn find_control(&self, arg: types::FindBy) -> Option<&dyn controls::Control> {
        for child in self.children.as_slice() {
            match arg {
                types::FindBy::Id(ref id) => {
                    if child.as_member().id() == *id {
                        return Some(child.as_ref());
                    }
                }
                types::FindBy::Tag(ref tag) => {
                    if let Some(mytag) = child.as_member().tag() {
                        if tag.as_str() == mytag {
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

impl ControlInner for CocoaList {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        let mut y = y;

        let self2 = unsafe { common::member_from_cocoa_id_mut::<List>(self.base.control).unwrap() };
        for ref mut child in self.children.as_mut_slice() {
            unsafe {
                let () = msg_send![self2.as_inner_mut().as_inner_mut().as_inner_mut().base.control, addSubview: child.native_id() as cocoa_id];
            }
            child.on_added_to_container(self2, x, y, control.measured.0, control.measured.1);
            let (_, yy) = child.size();
            y += yy as i32;
        }
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        let ll2: &List = unsafe { common::member_from_cocoa_id(self.base.control).unwrap() };
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

        fill_from_markup_base!(self, base, markup, registry, List, [MEMBER_TYPE_LINEAR_LAYOUT]);
        fill_from_markup_children!(self, base, markup, registry);
    }
}

impl HasLayoutInner for CocoaList {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl HasNativeIdInner for CocoaList {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaList {
    fn on_size_set(&mut self, base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        use plygui_api::controls::HasLayout;

        let this = base.as_any_mut().downcast_mut::<List>().unwrap();
        this.set_layout_width(layout::Size::Exact(width));
        this.set_layout_width(layout::Size::Exact(height));
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaList {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaList {}

impl Drawable for CocoaList {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);


    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let mut measured = false;
                let w = match control.layout.width {
                    layout::Size::Exact(w) => w,
                    layout::Size::MatchParent => parent_width,
                    layout::Size::WrapContent => {
                        let mut w = 0;
                        for child in self.children.as_mut_slice() {
                            let (cw, _, _) = child.measure(max(0, parent_width as i32) as u16, max(0, parent_height as i32) as u16);
                            w = max(w, cw);
                        }
                        measured = true;
                        max(0, w as i32) as u16
                    }
                };
                let h = match control.layout.height {
                    layout::Size::Exact(h) => h,
                    layout::Size::MatchParent => parent_height,
                    layout::Size::WrapContent => {
                        let mut h = 0;
                        for child in self.children.as_mut_slice() {
                            let ch = if measured {
                                child.size().1
                            } else {
                                let (_, ch, _) = child.measure(max(0, parent_width as i32) as u16, max(0, parent_height as i32) as u16);
                                ch
                            };
                            h += ch;
                        }
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
extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<List>(this).unwrap();
        let () = msg_send![super(sp.as_inner_mut().as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        sp.call_on_size(param.width as u16, param.height as u16)
    }
}
/*
#[allow(dead_code)]
pub(crate) fn spawn() -> Box<dyn controls::Control> {
    List::with_orientation(layout::Orientation::Vertical).into_control()
}
*/
extern "C" fn datasource_len(this: &mut Object, _: Sel, _: cocoa_id) -> NSInteger {
    unsafe {
        let sp = common::member_from_cocoa_id::<List>(this).unwrap();
        sp.as_inner().as_inner().base().adapter.len() as i64
    }
}
extern "C" fn spawn_item(this: &mut Object, _: Sel, _: cocoa_id, _: cocoa_id, row: NSInteger) -> cocoa_id {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<List>(this).unwrap();
        let sp2 = common::member_from_cocoa_id_mut::<List>(this).unwrap();
        let view = sp.as_inner_mut().as_inner_mut().base_mut().adapter.spawn_item_view(row as usize, sp2);
        sp.as_inner_mut().as_inner_mut().as_inner_mut().children.insert(row as usize, view);
        sp.as_inner_mut().as_inner_mut().as_inner_mut().children[row as usize].native_id() as cocoa_id
    }
}

default_impls_as!(List);
