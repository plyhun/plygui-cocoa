use crate::common::{self, *};
use cocoa::appkit::NSViewHeightSizable;

lazy_static! {
    static ref WINDOW_CLASS_INNER: common::RefClass = unsafe {
        register_window_class("PlyguiListInner", "NSTableView", |decl| {
            decl.add_method(sel!(validateProposedFirstResponder:forEvent:), validate_proposed_first_responder as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id) -> BOOL);
            decl.add_method(sel!(itemClicked:), item_clicked as extern "C" fn(&mut Object, Sel, cocoa_id));
            decl.add_method(sel!(numberOfRowsInTableView:), datasource_len as extern "C" fn(&mut Object, Sel, cocoa_id) -> NSInteger);
            decl.add_method(sel!(tableView:heightOfRow:), get_item_height as extern "C" fn(&mut Object, Sel, cocoa_id, NSInteger) -> f64);
            decl.add_method(sel!(tableView:viewForTableColumn:row:), spawn_item as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id, NSInteger) -> cocoa_id);
        })
    };
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiList", "NSScrollView", |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

pub type List = Member<Control<Adapter<CocoaList>>>;

#[repr(C)]
pub struct CocoaList {
    base: common::CocoaControlBase<List>,
    table: cocoa_id,
    items: Vec<Box<dyn controls::Control>>,
}

impl CocoaList {
    fn add_item_inner(&mut self, base: &mut MemberBase, i: usize) {
        let (member, control, adapter) = List::adapter_base_parts_mut(base);
        let (pw, ph) = control.measured;
        let this: &mut List = unsafe { utils::base_to_impl_mut(member) };
        
        let mut item = adapter.adapter.spawn_item_view(i, this);
        item.on_added_to_container(this, 0, 0, utils::coord_to_size(pw as i32) as u16, utils::coord_to_size(ph as i32) as u16);
                
        self.items.insert(i, item);
    }
    fn remove_item_inner(&mut self, base: &mut MemberBase, i: usize) {
        let this: &mut List = unsafe { utils::base_to_impl_mut(base) };
        self.items.remove(i).on_removed_from_container(this); 
    }    
}

impl AdapterViewInner for CocoaList {
    fn with_adapter(adapter: Box<dyn types::Adapter>) -> Box<List> {
        let base = common::CocoaControlBase::with_params(*WINDOW_CLASS);
        let base_bounds: NSRect = unsafe { msg_send![base.control, bounds] };
        let mut ll = Box::new(Member::with_inner(
            Control::with_inner(
                Adapter::with_inner(
                    CocoaList {
                        base: base,
                        table: unsafe {
                            let mut control: cocoa_id = msg_send![WINDOW_CLASS_INNER.0, alloc];
                            control = msg_send![control, initWithFrame: base_bounds];
                            control
                        },
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
            let table = ll.as_inner_mut().as_inner_mut().as_inner_mut().table;
            (&mut *table).set_ivar(common::IVAR, selfptr);
            
            let column: cocoa_id = msg_send![Class::get("NSTableColumn").unwrap(), alloc];
            let ident = NSString::alloc(nil).init_str("_");
            let column: cocoa_id = msg_send![column, initWithIdentifier:ident];
            let () = msg_send![table, addTableColumn: column];
            
            let _ = msg_send![table, setTarget: table];
            let _ = msg_send![table, setAction: sel!(itemClicked:)];
            let _ = msg_send![table, setFocusRingType:1 as NSUInteger];
        	let _ = msg_send![table, setHeaderView: nil];
        	
            let _ = msg_send![control, setAutohidesScrollers: NO];
            let _ = msg_send![control, setHasHorizontalScroller: NO];
            let _ = msg_send![control, setHasVerticalScroller: YES];            
            let _ = msg_send![control, setPostsFrameChangedNotifications: YES];
            let _ = msg_send![control, setAutoresizesSubviews:YES];
            let _ = msg_send![control, setAutoresizingMask: NSViewHeightSizable];
            
            let _ = msg_send![control, setDocumentView: table];
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
            let () = msg_send![self.table, reloadData];
        }
        self.base.invalidate();
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
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        unsafe { (&mut *self.table).set_ivar(common::IVAR_PARENT, parent.native_id() as *mut c_void); }
        self.measure(member, control, pw, ph);
        control.coords = Some((x, y));
        
        let (member, _, adapter) = List::adapter_base_parts_mut(member);

        for i in 0..adapter.adapter.len() {
            self.add_item_inner(member, i);
        }

        unsafe {                            
            let () = msg_send![self.table, setDelegate: self.table];
            let () = msg_send![self.table, setDataSource: self.table];
        }
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        unsafe {
            (&mut *self.table).set_ivar(common::IVAR_PARENT, ptr::null_mut::<c_void>());
            let () = msg_send![self.table, setDelegate: nil];
            let () = msg_send![self.table, setDataSource: nil];
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
        use plygui_api::markup::MEMBER_TYPE_LIST;

        fill_from_markup_base!(self, base, markup, registry, List, [MEMBER_TYPE_LIST]);
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
        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => {
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width,
                    layout::Size::Exact(w) => w,
                    layout::Size::WrapContent => defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING,
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height,
                    layout::Size::Exact(h) => h,
                    layout::Size::WrapContent => defaults::THE_ULTIMATE_ANSWER_TO_EVERYTHING,
                };
                (cmp::max(0, w as i32) as u16, cmp::max(0, h as i32) as u16)
            }
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

impl Drop for CocoaList {
    fn drop(&mut self) {
        let ll: &List = unsafe { common::member_from_cocoa_id(self.base.control).unwrap() };
        for ref mut child in self.items.as_mut_slice() {
            child.on_removed_from_container(ll);
        }
        unsafe {
            let () = msg_send![self.table, release];
        }
    }
}

extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<List>(this).unwrap();
        if let Some(cls) = Class::get("NSScrollView") {
            let () = msg_send![super(sp.as_inner_mut().as_inner_mut().as_inner_mut().base.control, cls), setFrameSize: param];
            sp.call_on_size(param.width as u16, param.height as u16);
        }
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
    let sp = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    unsafe { sp.as_inner_mut().as_inner_mut().as_inner_mut().items[row as usize].native_id() as cocoa_id }
}
extern "C" fn get_item_height(this: &mut Object, _: Sel, _: cocoa_id, row: NSInteger) -> f64 {
    let sp = unsafe { common::member_from_cocoa_id::<List>(this).unwrap() };
    let (_, h) = sp.as_inner().as_inner().as_inner().items[row as usize].size();
    h as f64
}
extern "C" fn item_clicked(this: &mut Object, _: Sel, _: cocoa_id) {
    let sp = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    let sp2 = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    let i: NSInteger = unsafe { msg_send![this, clickedRow] };
    if i < 0 {
        return;
    }
    let item_view = sp.as_inner_mut().as_inner_mut().as_inner_mut().items.get_mut(i as usize).unwrap();
    if let Some(ref mut callback) = sp2.as_inner_mut().as_inner_mut().base_mut().on_item_click {
        let sp2 = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
        (callback.as_mut())(sp2, i as usize, item_view.as_mut());
    }
}
extern "C" fn validate_proposed_first_responder(_: &mut Object, _: Sel, responder: cocoa_id, evt: cocoa_id) -> BOOL {
    let evt_type: NSEventType = unsafe { evt.eventType() };
    match evt_type {
        NSEventType::NSLeftMouseUp => unsafe { msg_send![responder, isKindOfClass: class!(NSButton)] },
        _ => NO //unsafe { msg_send![super(this, Class::get(BASE_CLASS).unwrap()), validateProposedFirstResponder:responder forEvent:evt] }
    }
}

default_impls_as!(List);
