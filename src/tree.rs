use crate::common::{self, *};
use cocoa::appkit::NSViewHeightSizable;

const BASE_CLASS: &str = "NSOutlineView";

lazy_static! {
    static ref WINDOW_CLASS_INNER: common::RefClass = unsafe {
        register_window_class("PlyguiTreeInner", BASE_CLASS, |decl| {
             decl.add_method(sel!(validateProposedFirstResponder:forEvent:), validate_proposed_first_responder as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id) -> BOOL);
             decl.add_method(sel!(outlineViewSelectionDidChange:), item_clicked as extern "C" fn(&mut Object, Sel, cocoa_id));
             decl.add_method(sel!(outlineView:heightOfRowByItem:), get_item_height as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id) -> f64);
             decl.add_method(sel!(outlineView:numberOfChildrenOfItem:), children_len as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id) -> NSInteger);
             decl.add_method(sel!(outlineView:isItemExpandable:), has_children as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id) -> BOOL);
             decl.add_method(sel!(outlineView:child:ofItem:), child_at as extern "C" fn(&mut Object, Sel, cocoa_id, NSInteger, cocoa_id) -> cocoa_id);
             //decl.add_method(sel!(outlineView:shouldCollapseItem:), should_collapse as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id) -> BOOL);
             decl.add_method(sel!(outlineView:shouldExpandItem:), should_expand as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id) -> BOOL);
             decl.add_method(sel!(outlineView:viewForTableColumn:item:), spawn_item as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id, cocoa_id) -> cocoa_id);
        })
    };
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiTree", "NSScrollView", |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
    //NSOutliveViewDelegate REQUIRES a datasource to be the ObjC object instance.
    static ref NODE_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiTree1Node", "NSObject", |decl| {
            decl.add_ivar::<BOOL>("expanded");
            decl.add_ivar::<*mut c_void>("root");
            decl.add_ivar::<cocoa_id>("native");
            decl.add_ivar::<cocoa_id>("branches");
        })
    };
}

pub type Tree = AMember<AControl<AContainer<AAdapted<ATree<CocoaTree>>>>>;

#[repr(C)]
pub struct CocoaTree {
    base: common::CocoaControlBase<Tree>,
    table: cocoa_id,
    items: cocoa_id,
    on_item_click: Option<callbacks::OnItemClick>
}

impl CocoaTree {
    fn add_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize], node: &adapter::Node) {
        let (member, control, adapter, _) = unsafe { Tree::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        let this: &mut Tree = unsafe { utils::base_to_impl_mut(member) };
        
        let mut view = adapter.adapter.spawn_item_view(indexes, this).unwrap();
        view.on_added_to_container(this, 0, 0, utils::coord_to_size(pw as i32) as u16, utils::coord_to_size(ph as i32) as u16);
        let id = unsafe { view.native_id() };
        let view = Box::into_raw(Box::new(view));
        let mut items = self.items;
        unsafe {
            for i in 0..indexes.len() {
                let index = indexes[i];
                let end = i+1 >= indexes.len();
                if end {
                    let item: cocoa_id = msg_send![NODE_CLASS.0, alloc];
                    let branches: cocoa_id = msg_send![class!(NSMutableArray), alloc];
                    let branches: cocoa_id = msg_send![branches, init];
                    (&mut *item).set_ivar("expanded", if let adapter::Node::Branch(expanded) = node { if *expanded { YES } else { NO } } else { NO });
                	(&mut *item).set_ivar("branches", branches);
                	(&mut *item).set_ivar("native", id as cocoa_id);
                	(&mut *item).set_ivar("root", view as *mut c_void);
                    let () = msg_send![items, insertObject:item atIndex:index];
                } else {
                	items = msg_send![items, objectAtIndex:index];
                	items = *(&mut *items).get_ivar::<cocoa_id>("branches");
                }
            }
        }
    }
    fn remove_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize]) {
        let this: &mut Tree = unsafe { utils::base_to_impl_mut(base) };
        let mut items = self.items;
        for i in 0..indexes.len() {
            let index = indexes[i];    
            unsafe {
                if i+1 >= indexes.len() {
                    let deleted: cocoa_id = msg_send![items, objectAtIndex:index];
                    remove_item(deleted, index, items, this);
                } else {
                    items = msg_send![items, objectAtIndex:index];
                	items = *(&mut *items).get_ivar::<cocoa_id>("branches");
                }
            }
        }
    }
    fn update_item_inner(&mut self, base: &mut MemberBase, indexes: &[usize], node: &adapter::Node) {
    	let (member, control, adapter, _) = unsafe { Tree::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        let this: &mut Tree = unsafe { utils::base_to_impl_mut(member) };
        
        let mut view = adapter.adapter.spawn_item_view(indexes, this);
		let id = view.as_ref().map(|item1| unsafe { item1.native_id() }).unwrap();
			        
		let mut items = self.items;
        for i in 0..indexes.len() {
            let index = indexes[i];    
            unsafe {
                if i+1 >= indexes.len() {
                    let deleted: cocoa_id = msg_send![items, objectAtIndex:index];
                    remove_item(deleted, index, items, this);
    	            (&mut *deleted).set_ivar("expanded", if let adapter::Node::Branch(expanded) = node { if *expanded { YES } else { NO } } else { NO });
                	(&mut *deleted).set_ivar("native", id);
                	(&mut *deleted).set_ivar("root", Box::into_raw(Box::new(view.as_mut().map(|view| {
                    	            view.on_added_to_container(this, 0, 0, utils::coord_to_size(pw as i32) as u16, utils::coord_to_size(ph as i32) as u16);
                                    view
                	            }).take().unwrap())) as *mut c_void);
                } else {
                	items = msg_send![items, objectAtIndex:index];
                	items = *(&mut *items).get_ivar::<cocoa_id>("branches");
                }
            }
        }
    }  
}

impl<O: controls::Tree> NewTreeInner<O> for CocoaTree {
    fn with_uninit(ptr: &mut mem::MaybeUninit<O>) -> Self {
        let base = common::CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>);
        let base_bounds: NSRect = unsafe { msg_send![base.control, bounds] };
        let items: cocoa_id = unsafe { msg_send![class!(NSMutableArray), alloc] };
        let items: cocoa_id = unsafe { msg_send![items, init] };
        let li = CocoaTree {
            base: base,
            table: unsafe {
                let mut control: cocoa_id = msg_send![WINDOW_CLASS_INNER.0, alloc];
                control = msg_send![control, initWithFrame: base_bounds];
                control
            },
            on_item_click: None,
            items: items,
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
            let () = msg_send![table, setOutlineTableColumn: column];
            let () = msg_send![table, setTarget: table];
            //let () = msg_send![table, setAction: sel!(outlineViewSelectionDidChange:)];
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
impl TreeInner for CocoaTree {
    fn with_adapter(adapter: Box<dyn types::Adapter>) -> Box<dyn controls::Tree> {
        let mut b: Box<mem::MaybeUninit<Tree>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AAdapted::with_inner(
                        ATree::with_inner(
                            <Self as NewTreeInner<Tree>>::with_uninit(b.as_mut())
                        ),
                        adapter,
                        &mut b,
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
impl ItemClickableInner for CocoaTree {
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
impl AdaptedInner for CocoaTree {
    fn on_item_change(&mut self, base: &mut MemberBase, value: adapter::Change) {
        match value {
            adapter::Change::Added(at, ref node) => {
                self.add_item_inner(base, at, node);
            },
            adapter::Change::Removed(at) => {
                self.remove_item_inner(base, at);
            },
            adapter::Change::Edited(at, ref node) => {
                self.update_item_inner(base, at, node);
            },
        }
        unsafe {
            let () = msg_send![self.table, reloadData];
        }
        self.base.invalidate();
    }
}

impl ContainerInner for CocoaTree {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        fn find_control_inner_mut<'a>(vec: cocoa_id, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
            let len = unsafe { msg_send![vec, count] };
            for i in 0..len {
                let child: cocoa_id = unsafe { msg_send![vec, objectAtIndex:i] };
                let root = unsafe { &mut *(*(&mut *child).get_ivar::<*mut c_void>("root") as *mut Box<dyn controls::Control>) };
                match arg {
                    types::FindBy::Id(id) => {
                        if root.as_member_mut().id() == id {
                            return Some(root.as_mut());
                        }
                    }
                    types::FindBy::Tag(tag) => {
                        if let Some(mytag) = root.as_member_mut().tag() {
                            if tag == mytag {
                                return Some(root.as_mut());
                            }
                        }
                    }
                }
                if let Some(c) = root.is_container_mut() {
                    let ret = c.find_control_mut(arg);
                    if ret.is_some() {
                        return ret;
                    }
                }
                let branches = unsafe { *(&mut *child).get_ivar::<cocoa_id>("branches") };
                let ret = find_control_inner_mut(branches, arg);
                if ret.is_some() {
                    return ret;
                }
            }
            None
        }
        find_control_inner_mut(self.items, arg)
    }
    fn find_control<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
        fn find_control_inner<'a>(vec: cocoa_id, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
            let len = unsafe { msg_send![vec, count] };
            for i in 0..len {
                let child: cocoa_id = unsafe { msg_send![vec, objectAtIndex:i] };
                let root = unsafe { &*(*(&mut *child).get_ivar::<*mut c_void>("root") as *mut Box<dyn controls::Control>) };
                match arg {
                    types::FindBy::Id(id) => {
                        if root.as_member().id() == id {
                            return Some(root.as_ref());
                        }
                    }
                    types::FindBy::Tag(tag) => {
                        if let Some(mytag) = root.as_member().tag() {
                            if tag == mytag {
                                return Some(root.as_ref());
                            }
                        }
                    }
                }
                if let Some(c) = root.is_container() {
                    let ret = c.find_control(arg);
                    if ret.is_some() {
                        return ret;
                    }
                }
                let branches = unsafe { *(&*child).get_ivar::<cocoa_id>("branches") };
                let ret = find_control_inner(branches, arg);
                if ret.is_some() {
                    return ret;
                }
            }
            None
        }
        find_control_inner(self.items, arg)
    }
    fn native_container_id(&self) -> Self::Id {
        self.table.into()
    }
}

impl ControlInner for CocoaTree {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        unsafe { (&mut *self.table).set_ivar(common::IVAR_PARENT, parent.native_id() as *mut c_void); }
        self.measure(member, control, pw, ph);
        control.coords = Some((x, y));
        
        let (member, _, adapter, _) = unsafe { Tree::adapter_base_parts_mut(member) };

        adapter.adapter.for_each(&mut (|indexes, node| {
            self.add_item_inner(member, indexes, node);
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
        let this: &mut Tree = unsafe { common::member_from_cocoa_id_mut(self.base.control).unwrap() };
        let len = unsafe { msg_send![self.items, count] };
        for i in 0..len {
            let child: cocoa_id = unsafe { msg_send![self.items, objectAtIndex:i] };
            unsafe { remove_item(child, i, self.items, this); }
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
        use plygui_api::markup::MEMBER_TYPE_TREE;

        fill_from_markup_base!(self, base, markup, registry, Tree, [MEMBER_TYPE_TREE]);
        //fill_from_markup_items!(self, base, markup, registry);
    }
}

impl HasLayoutInner for CocoaTree {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl HasNativeIdInner for CocoaTree {
    type Id = common::CocoaId;

    fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaTree {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaTree {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaTree {}

impl Drawable for CocoaTree {
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

impl Drop for CocoaTree {
    fn drop(&mut self) {
        unsafe {
            let this: &mut Tree = common::member_from_cocoa_id_mut(self.base.control).unwrap();
            let len = msg_send![self.items, count];
            for i in (0..len).rev() {
                let child: cocoa_id = msg_send![self.items, objectAtIndex:i];
                remove_item(child, i, self.items, this);
            }
            let () = msg_send![self.items, release];
            let () = msg_send![self.table, release];
        }
    }
}

extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = common::member_from_cocoa_id_mut::<Tree>(this).unwrap();
        let b2 = common::member_from_cocoa_id_mut::<Tree>(this).unwrap();
        (b.inner().inner().inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: controls::Tree>(this: &mut Tree, _: Sel, param: NSSize) {
    unsafe {
        if let Some(cls) = Class::get("NSScrollView") {
            let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, cls), setFrameSize: param];
            this.call_on_size::<O>(param.width as u16, param.height as u16);
        }
    }
}
impl Spawnable for CocoaTree {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_adapter(Box::new(types::imp::StringVecAdapter::<crate::imp::Text>::new())).into_control()
    }
}

extern "C" fn children_len(this: &mut Object, _:Sel, _:cocoa_id, item: cocoa_id) -> NSInteger {
    println!(" == len at {:?}", item);
    let items = if nil == item { 
        let sp = unsafe { common::member_from_cocoa_id_mut::<Tree>(this).unwrap() };
        let tree = sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
        tree.items 
    } else { 
        unsafe { *(&mut *item).get_ivar::<cocoa_id>("branches") }
    };
    let len: NSInteger = unsafe { msg_send![items, count] };
    println!(" === is {}", len);
    len
}
extern "C" fn has_children(_: &mut Object, _:Sel, _:cocoa_id, item: cocoa_id) -> BOOL {
    println!(" == has at {:?}", item);
    let branches = unsafe { *(&mut *item).get_ivar::<cocoa_id>("branches") };
    let len: NSInteger = unsafe { msg_send![branches, count] };
    if len > 0 { YES } else { NO }
}
extern "C" fn should_expand(_: &mut Object, _:Sel, _:cocoa_id, item: cocoa_id) -> BOOL {
    println!(" == expand {:?}", item);
    unsafe { *(&mut *item).get_ivar::<BOOL>("expanded") }
}
extern "C" fn child_at(this: &mut Object, _:Sel, _:cocoa_id, index: NSInteger, item: cocoa_id) -> cocoa_id {
    println!(" == child at {:?} / {}", item, index);
    let items = if nil == item { 
        let sp = unsafe { common::member_from_cocoa_id_mut::<Tree>(this).unwrap() };
        let tree = sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
        tree.items 
    } else { 
        unsafe { *(&mut *item).get_ivar::<cocoa_id>("branches") }
    };
    let child: cocoa_id = unsafe { msg_send![items, objectAtIndex:index] };
    println!(" === is {:?}", child);
    child
}
extern "C" fn spawn_item(_: &mut Object, _:Sel, _:cocoa_id, _column: cocoa_id, item: cocoa_id) -> cocoa_id {
    println!(" == spawn at {:?}", item);
    unsafe { *(&mut *item).get_ivar::<cocoa_id>("native") }
}
extern "C" fn get_item_height(_: &mut Object, _: Sel, _: cocoa_id, item: cocoa_id) -> f64 {
    println!(" == height at {:?}", item);
    let root = unsafe { &mut *(*(&mut *item).get_ivar::<*mut c_void>("root") as *mut Box<dyn controls::Control>) };
    let (_, h) = root.size();
    h as f64
}
extern "C" fn item_clicked(this: &mut Object, _: Sel, item: cocoa_id) {
    println!(" == clicked at {:?}", item);
    let sp2 = unsafe { common::member_from_cocoa_id_mut::<Tree>(this).unwrap() };
    let i: NSInteger = unsafe { msg_send![this, selectedRow] };
    if i < 0 {
        return;
    }
    let item: cocoa_id = unsafe { msg_send![this, itemAtRow: i] };
    if nil == item {
        panic!("No item at clicked index: {}", i);
    }
    let root = unsafe { &mut *(*(&mut *item).get_ivar::<*mut c_void>("root") as *mut Box<dyn controls::Control>) };
    if let Some(ref mut callback) = sp2.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().on_item_click {
        let sp2 = unsafe { common::member_from_cocoa_id_mut::<Tree>(this).unwrap() };
        (callback.as_mut())(sp2, &[i as usize], root.as_mut());
    }
}
extern "C" fn validate_proposed_first_responder(this: &mut Object, sel: Sel, responder: cocoa_id, evt: cocoa_id) -> BOOL {
    let evt_type: NSEventType = unsafe { evt.eventType() };
    match evt_type {
        NSEventType::NSLeftMouseDown => unsafe { 
            let is_button: BOOL = msg_send![responder, isKindOfClass: class!(NSButton)];
//            if NO == is_button {
//                item_clicked(this, sel, responder);
//                msg_send![super(this, Class::get(BASE_CLASS).unwrap()), validateProposedFirstResponder:responder forEvent:evt]
//            } else {
                is_button
//            }
        },
        _ => unsafe { msg_send![super(this, Class::get(BASE_CLASS).unwrap()), validateProposedFirstResponder:responder forEvent:evt] }
    }
}
unsafe fn remove_item(item: cocoa_id, index: usize, parent: cocoa_id, this: &mut Tree) {
    let branches = *(&mut *item).get_ivar::<cocoa_id>("branches");
    let len = msg_send![branches, count];
    for i in (0..len).rev() {
        let child: cocoa_id = msg_send![branches, objectAtIndex:index];
        remove_item(child, i, branches, this);
    }
    let root = *(&mut *item).get_ivar::<*mut c_void>("root") as *mut Box<dyn controls::Control>;
    let mut root = Box::from_raw(root);
    root.on_removed_from_container(this);
    let () = msg_send![parent, removeObjectAtIndex:index];
    let () = msg_send![branches, release];
}
