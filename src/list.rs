use crate::common::{self, *};
use cocoa::appkit::NSViewHeightSizable;

const BASE_CLASS: &str = "NSTableView";

lazy_static! {
    static ref WINDOW_CLASS_INNER: common::RefClass = unsafe {
        register_window_class("PlyguiListInner", BASE_CLASS, |decl| {
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

pub type List = AMember<AControl<AContainer<AAdapted<AList<CocoaList>>>>>;

#[repr(C)]
pub struct CocoaList {
    base: common::CocoaControlBase<List>,
    table: cocoa_id,
    items: Vec<Box<dyn controls::Control>>,
    on_item_click: Option<callbacks::OnItemClick>
}

impl CocoaList {
    fn add_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize]) {
        let i = indexes[0];
        let (member, control, adapter, _) = unsafe { List::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        let this: &mut List = unsafe { utils::base_to_impl_mut(member) };
        
        let mut item = adapter.adapter.spawn_item_view(indexes, this).unwrap();
        item.on_added_to_container(this, 0, 0, utils::coord_to_size(pw as i32) as u16, utils::coord_to_size(ph as i32) as u16);
                
        self.items.insert(i, item);
    }
    fn remove_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize]) {
        let this: &mut List = unsafe { utils::base_to_impl_mut(base) };
        self.items.remove(indexes[0]).on_removed_from_container(this); 
    }    
}

impl<O: controls::List> NewListInner<O> for CocoaList {
    fn with_uninit(ptr: &mut mem::MaybeUninit<O>) -> Self {
        let base = common::CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>);
        let base_bounds: NSRect = unsafe { msg_send![base.control, bounds] };
        let li = CocoaList {
            base: base,
            table: unsafe {
                let mut control: cocoa_id = msg_send![WINDOW_CLASS_INNER.0, alloc];
                control = msg_send![control, initWithFrame: base_bounds];
                control
            },
            on_item_click: None,
            items: Vec::new(),
        };
        let selfptr = ptr as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            let control = li.base.control;
            (&mut *control).set_ivar(common::IVAR, selfptr);
            let table = li.table;
            (&mut *table).set_ivar(common::IVAR, selfptr);
            
            let column: cocoa_id = msg_send![Class::get("NSTableColumn").unwrap(), alloc];
            let ident = NSString::alloc(nil).init_str("_");
            let column: cocoa_id = msg_send![column, initWithIdentifier:ident];
            let () = msg_send![table, addTableColumn: column];
            
            let () = msg_send![table, setTarget: table];
            let () = msg_send![table, setAction: sel!(itemClicked:)];
            let () = msg_send![table, setFocusRingType:1 as NSUInteger];
            	let () = msg_send![table, setHeaderView: nil];
        	
            let () = msg_send![control, setAutohidesScrollers: NO];
            let () = msg_send![control, setHasHorizontalScroller: NO];
            let () = msg_send![control, setHasVerticalScroller: YES];            
            let () = msg_send![control, setPostsFrameChangedNotifications: YES];
            let () = msg_send![control, setAutoresizesSubviews:YES];
            let () = msg_send![control, setAutoresizingMask: NSViewHeightSizable];
            
            let () = msg_send![control, setDocumentView: table];
        }
        li
    }
}
impl ListInner for CocoaList {
    fn with_adapter(adapter: Box<dyn types::Adapter>) -> Box<dyn controls::List> {
        let mut b: Box<mem::MaybeUninit<List>> = Box::new_uninit();
        let mut ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AAdapted::with_inner(
                        AList::with_inner(
                            <Self as NewListInner<List>>::with_uninit(b.as_mut())
                        ),
                        adapter,
                        &mut b,
                    ),
                )
            ),
        );
        ab.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().items = Vec::new();
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
}
impl ItemClickableInner for CocoaList {
    fn item_click(&mut self, indexes: &[usize], item_view: &mut dyn controls::Control, skip_callbacks: bool) {
        if !skip_callbacks{
            let self2 = self.base.as_outer_mut();
            if let Some(ref mut callback) = self.on_item_click {
                (callback.as_mut())(self2, indexes, item_view)
            }
        }
    }
    fn on_item_click(&mut self, callback: Option<callbacks::OnItemClick>) {
        self.on_item_click = callback;
    }
}
impl AdaptedInner for CocoaList {
    fn on_item_change(&mut self, base: &mut MemberBase, value: adapter::Change) {
        match value {
            adapter::Change::Added(at, _) => {
                self.add_item_inner(base, at);
            },
            adapter::Change::Removed(at) => {
                self.remove_item_inner(base, at);
            },
            adapter::Change::Edited(_,_) => {
            },
        }
        unsafe {
            let () = msg_send![self.table, reloadData];
        }
        self.base.invalidate();
    }
}

impl ContainerInner for CocoaList {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        for child in self.items.as_mut_slice() {
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
        for child in self.items.as_slice() {
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

impl ControlInner for CocoaList {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        unsafe { (&mut *self.table).set_ivar(common::IVAR_PARENT, parent.native_id() as *mut c_void); }
        self.measure(member, control, pw, ph);
        control.coords = Some((x, y));
        
        let (member, _, adapter, _) = unsafe { List::adapter_base_parts_mut(member) };

        adapter.adapter.for_each(&mut (|indexes, _node| {
            self.add_item_inner(member, indexes);
        }));
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

    fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaList {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
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

extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = common::member_from_cocoa_id_mut::<List>(this).unwrap();
        let b2 = common::member_from_cocoa_id_mut::<List>(this).unwrap();
        (b.inner().inner().inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: controls::List>(this: &mut List, _: Sel, param: NSSize) {
    unsafe {
        if let Some(cls) = Class::get("NSScrollView") {
            let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, cls), setFrameSize: param];
            this.call_on_size::<O>(param.width as u16, param.height as u16);
        }
    }
}
impl Spawnable for CocoaList {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_adapter(Box::new(types::imp::StringVecAdapter::<crate::imp::Text>::new())).into_control()
    }
}

extern "C" fn datasource_len(this: &mut Object, _: Sel, _: cocoa_id) -> NSInteger {
    unsafe {
        let sp = common::member_from_cocoa_id::<List>(this).unwrap();
        sp.inner().inner().inner().base.adapter.len_at(&[]).unwrap() as i64
    }
}
extern "C" fn spawn_item(this: &mut Object, _: Sel, _: cocoa_id, _: cocoa_id, row: NSInteger) -> cocoa_id {
    let sp = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    unsafe { sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().items[row as usize].native_id() as cocoa_id }
}
extern "C" fn get_item_height(this: &mut Object, _: Sel, _: cocoa_id, row: NSInteger) -> f64 {
    let sp = unsafe { common::member_from_cocoa_id::<List>(this).unwrap() };
    let (_, h) = sp.inner().inner().inner().inner().inner().items[row as usize].size();
    h as f64
}
extern "C" fn item_clicked(this: &mut Object, _: Sel, _: cocoa_id) {
    let sp = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    let sp2 = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
    let i: NSInteger = unsafe { msg_send![this, clickedRow] };
    if i < 0 {
        return;
    }
    let item_view = sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().items.get_mut(i as usize).unwrap();
    if let Some(ref mut callback) = sp2.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().on_item_click {
        let sp2 = unsafe { common::member_from_cocoa_id_mut::<List>(this).unwrap() };
        (callback.as_mut())(sp2, &[i as usize], item_view.as_mut());
    }
}
extern "C" fn validate_proposed_first_responder(this: &mut Object, _: Sel, responder: cocoa_id, evt: cocoa_id) -> BOOL {
    let evt_type: NSEventType = unsafe { evt.eventType() };
    match evt_type {
        NSEventType::NSLeftMouseUp => unsafe { msg_send![responder, isKindOfClass: class!(NSButton)] },
        _ => NO //unsafe { msg_send![super(this, Class::get(BASE_CLASS).unwrap()), validateProposedFirstResponder:responder forEvent:evt] }
    }
}
