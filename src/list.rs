use crate::common::{self, *};

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiList", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
            decl.add_method(sel!(itemClicked:), item_clicked as extern "C" fn(&mut Object, Sel, cocoa_id));
            decl.add_method(sel!(numberOfRowsInTableView:), datasource_len as extern "C" fn(&mut Object, Sel, cocoa_id) -> NSInteger);
            decl.add_method(sel!(tableView:heightOfRow:), get_item_height as extern "C" fn(&mut Object, Sel, cocoa_id, NSInteger) -> f64);
            decl.add_method(sel!(tableView:viewForTableColumn:row:), spawn_item as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id, NSInteger) -> cocoa_id);
        })
    };
}

const BASE_CLASS: &str = "NSTableView";

pub type List = Member<Control<Adapter<CocoaList>>>;

#[repr(C)]
pub struct CocoaList {
    base: common::CocoaControlBase<List>,
    items: Vec<Box<dyn controls::Control>>,
}

impl CocoaList {
    fn add_item_inner(&mut self, base: &mut MemberBase, i: usize) {
        let (member, control, adapter) = List::adapter_base_parts_mut(base);
        let (pw, ph) = control.measured;
        let this: &mut List = unsafe { utils::base_to_impl_mut(member) };
        
        let mut item = adapter.adapter.spawn_item_view(i, this);
        item.on_added_to_container(this, 0, 0, utils::coord_to_size(pw as i32) as u16, utils::coord_to_size(ph as i32) as u16);
                
        let (_, yy) = item.size();
        self.items.insert(i, item);
        
        unsafe {
            let () = msg_send![self.base.control, setRowHeight: yy as f64];
        }
    }
    fn remove_item_inner(&mut self, base: &mut MemberBase, i: usize) {
        let this: &mut List = unsafe { utils::base_to_impl_mut(base) };
        self.items.remove(i).on_removed_from_container(this); 
    }    
}

impl AdapterViewInner for CocoaList {
    fn with_adapter(adapter: Box<dyn types::Adapter>) -> Box<List> {
        let mut ll = Box::new(Member::with_inner(
            Control::with_inner(
                Adapter::with_inner(
                    CocoaList {
                        base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                        items: Vec::new(),
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
            
            let _ = msg_send![control, setTarget: control];
            let _ = msg_send![control, setAction: sel!(itemClicked:)];
        }
        ll
    }
    fn on_item_change(&mut self, base: &mut MemberBase, value: types::Change) {
        match value {
            types::Change::Added(at) => {
                self.add_item_inner(base, at);
            },
            types::Change::Removed(at) => {
                self.remove_item_inner(base, at);
            },
            types::Change::Edited(_) => {
            },
        }
        unsafe {
            let () = msg_send![self.base.control, reloadData];
        }
    }
}

impl ContainerInner for CocoaList {
    fn find_control_mut(&mut self, arg: types::FindBy) -> Option<&mut dyn controls::Control> {
        for child in self.items.as_mut_slice() {
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
        for child in self.items.as_slice() {
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
        control.coords = Some((x, y));
        
        let (member, _, adapter) = List::adapter_base_parts_mut(member);

        for i in 0..adapter.adapter.len() {
            self.add_item_inner(member, i);
        }

        unsafe {                            
            let () = msg_send![self.base.control, setDelegate: self.base.control];
            let () = msg_send![self.base.control, setDataSource: self.base.control];
        }
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        unsafe {
            let () = msg_send![self.base.control, setDelegate: nil];
            let () = msg_send![self.base.control, setDataSource: nil];
        }
        let ll2: &List = unsafe { common::member_from_cocoa_id(self.base.control).unwrap() };
        for ref mut child in self.items.as_mut_slice() {
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
        //fill_from_markup_items!(self, base, markup, registry);
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
                        for child in self.items.as_mut_slice() {
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
                        for child in self.items.as_mut_slice() {
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
        if let Some(cls) = Class::get(BASE_CLASS) {
            let () = msg_send![super(sp.as_inner_mut().as_inner_mut().as_inner_mut().base.control, cls), setFrameSize: param];
            sp.call_on_size(param.width as u16, param.height as u16);
        }
    }
    /*
    if let Some(sp) = unsafe { common::member_from_cocoa_id_mut::<List>(this) } {
        unsafe { let () = msg_send![super(sp.as_inner_mut().as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param]; }
        sp.call_on_size(param.width as u16, param.height as u16)
    }
    
    */
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
    let sp = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    println!("spawned {}", row);
    unsafe { sp.as_inner_mut().as_inner_mut().as_inner_mut().items[row as usize].native_id() as cocoa_id }
}
extern "C" fn get_item_height(this: &mut Object, _: Sel, _: cocoa_id, row: NSInteger) -> f64 {
    let sp = unsafe { common::member_from_cocoa_id::<List>(this).unwrap() };
    /*if let Some(item) = sp.as_inner().as_inner().as_inner().items.get(row as usize) {
        let (_, h) = item.size();
        h as f64
    } else {
        1.0
    }*/
    let (_, h) = sp.as_inner().as_inner().as_inner().items[row as usize].size();
    println!("height {} = {}", row, h);
    h as f64
}
extern "C" fn item_clicked(this: &mut Object, _: Sel, _: cocoa_id) {
    let sp = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    let sp2 = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    let i: NSInteger = unsafe { msg_send![this, clickedRow] };
    println!("clicked {}", i);
    if i < 0 {
        return;
    }
    let item_view = sp.as_inner_mut().as_inner_mut().as_inner_mut().items.get_mut(i as usize).unwrap();
    if let Some(ref mut callback) = sp2.as_inner_mut().as_inner_mut().base_mut().on_item_click {
        let sp2 = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
        (callback.as_mut())(sp2, i as usize, item_view.as_mut());
    }
}

default_impls_as!(List);
