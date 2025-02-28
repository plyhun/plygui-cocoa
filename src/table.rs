use crate::{common::{self, matrix::*, *}, table};
use cocoa::appkit::NSViewHeightSizable;
use cocoa::foundation::NSArray;

const BASE_CLASS: &str = "NSTableView";

lazy_static! {
    static ref WINDOW_CLASS_INNER: common::RefClass = unsafe {
        register_window_class("PlyguiTableInner", BASE_CLASS, |decl| {
            decl.add_method(sel!(validateProposedFirstResponder:forEvent:), validate_proposed_first_responder as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id) -> BOOL);
            decl.add_method(sel!(itemClicked:), item_clicked as extern "C" fn(&mut Object, Sel, cocoa_id));
            decl.add_method(sel!(numberOfRowsInTableView:), datasource_len as extern "C" fn(&mut Object, Sel, cocoa_id) -> NSInteger);
            decl.add_method(sel!(tableView:heightOfRow:), get_item_height as extern "C" fn(&mut Object, Sel, cocoa_id, NSInteger) -> f64);
            decl.add_method(sel!(tableView:viewForTableColumn:row:), spawn_item as extern "C" fn(&mut Object, Sel, cocoa_id, cocoa_id, NSInteger) -> cocoa_id);
        })
    };
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiTable", "NSScrollView", |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

pub type Table = AMember<AControl<AContainer<AAdapted<ATable<CocoaTable>>>>>;

#[repr(C)]
pub struct CocoaTable {
    base: common::CocoaControlBase<Table>,
    table: cocoa_id,
    header: cocoa_id,
    data: Matrix<cocoa_id>,
    on_item_click: Option<callbacks::OnItemClick>
}

impl CocoaTable {
    fn add_row_inner(&mut self, base: &mut MemberBase, index: usize) -> Option<&mut Row<cocoa_id>> {
        let (_, control, _, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let row = unsafe {
            Row {
                cells: self.data.cols.iter_mut().map(|_| None).collect(),
                native: index as cocoa_id,
                control: None,
                height: self.data.default_row_height,
            }
        };
        self.data.rows.insert(index, row);
        self.resize_row(control, index, self.data.default_row_height, true);
        unsafe {
            let () = msg_send![self.table, reloadData];
        }
        self.data.row_at_mut(index)
    }
    fn remove_row_inner(&mut self, base: &mut MemberBase, index: usize) {
        let (member, control, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        let table = self.table.clone();
        self.data.row_at_mut(index).map(|row| {
            (0..row.cells.len()).into_iter().for_each(|y| {
                row.cells.remove(y).and_then(|mut cell| cell.control.map(|mut ctl| ctl.on_removed_from_container(this)));
            });
            unsafe {
                let () = msg_send![table, reloadData];
            }
        });
    }
	fn add_column_inner(&mut self, base: &mut MemberBase, index: usize) {
        let (member, control, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        let width = utils::coord_to_size(pw as i32);
        let height = utils::coord_to_size(ph as i32);
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        let indices = &[index];
        let mut item: Option<Box<dyn controls::Control>> = adapter.adapter.spawn_item_view(indices, this);
        let maybe_title = item.as_mut().map(|item| {
            item.set_layout_width(layout::Size::Exact(width));
            item.set_layout_height(self.data.default_row_height);
            item.on_added_to_container(this, 0, 0, width, height);
            item.is_has_label().map(|has_label| has_label.label())
        }).flatten();
        let native = unsafe { 
            let column: cocoa_id = msg_send![Class::get("NSTableColumn").unwrap(), alloc];
            let ident = NSString::alloc(nil).init_str(maybe_title.as_ref().map(|title| title.as_ref()).unwrap_or("-"));
            let column: cocoa_id = msg_send![column, initWithIdentifier:ident];
            let () = msg_send![self.table, addTableColumn: column];
            column
        };
        self.data.cols.insert(index, Column {
            control: item,
            native: native,
            width: layout::Size::MatchParent,
        });
        self.resize_column(control, index, self.data.cols[index].width);
        self.data.rows.iter_mut().enumerate().for_each(|(row_index, row)| {
            row.cells.insert(index, None);
            this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().resize_row(control, row_index, row.height, true);
        });
        unsafe {
            let () = msg_send![self.table, reloadData];
        }
        //self.redraw_column_labels(member);
    }
	fn add_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        let (member, control, adapter, _) = unsafe { Table::adapter_base_parts_mut(base) };
        let (pw, ph) = control.measured;
        if self.data.rows.len() <= y {
            self.add_row_inner(member, y);
        }
        if self.data.cols.len() <= x {
            self.add_column_inner(member, x);
        }
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        adapter.adapter.spawn_item_view(&[x, y], this).map(|mut item| {
            let table_id = self.table.clone();
            let width: f32 = unsafe { 
                let columns: cocoa_id = msg_send![table_id, tableColumns];
                let column: cocoa_id = NSArray::objectAtIndex(columns, x as u64);
                msg_send![column, width]
            };
            self.data.rows.get_mut(y).map(|row| {
                item.set_layout_width(layout::Size::Exact(width as u16));
                item.set_layout_height(row.height);
                item.on_added_to_container(this, 0, 0, pw, ph);
                
                row.cells.insert(x, Some(Cell {
                    native: unsafe { item.native_id() as cocoa_id },
                    control: Some(item),
                }));
                if row.cells.len() > x {
                    // facepalm
                    row.cells.remove(x+1);
                }
            });
            unsafe {
                let () = msg_send![self.table, reloadData];
            }
        });
    }
	fn remove_column_inner(&mut self, member: &mut MemberBase, index: usize) {
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        let widget = &self.base.control;
        self.data.rows.iter_mut().enumerate().for_each(|(row_index, row)| {
            //this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().remove_cell_inner(member, row_index, index);
            let mut cell = if index < row.cells.len() { row.cells.remove(index) } else { None };
            cell.map(|cell| {
                cell.control.map(|mut control| control.on_removed_from_container(this));
            });
        });
        let column = if index < self.data.cols.len() { Some(self.data.cols.remove(index)) } else { None };
        column.map(|column| {
            column.control.map(|mut column| column.on_removed_from_container(this));
            unsafe {
                let () = msg_send![self.table, reloadData];
            }
        });
    }
    fn remove_cell_inner(&mut self, member: &mut MemberBase, x: usize, y: usize) {
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        let table = self.table.clone();
        self.data.rows.get_mut(y).map(|row| {
            row.cells.remove(x).map(|mut cell| {
                cell.control.as_mut().map(|mut control| control.on_removed_from_container(this));
            });
            row.cells.insert(x, None);
            unsafe {
                let () = msg_send![table, reloadData];
            }
        });
    }
    fn change_column_inner(&mut self, base: &mut MemberBase, index: usize) {
        self.remove_column_inner(base, index);
        self.add_column_inner(base, index);
    }
    fn change_cell_inner(&mut self, base: &mut MemberBase, x: usize, y: usize) {
        self.remove_cell_inner(base, x, y);
        self.add_cell_inner(base, x, y);
    }
    fn resize_row(&mut self, base: &ControlBase, index: usize, size: layout::Size, force: bool) {
        let (w, h) = base.measured;
        let height = match size {
            layout::Size::Exact(height) => height,
            layout::Size::WrapContent => self.data.rows.iter()
                    .flat_map(|row| row.cells.iter())
                    .filter(|cell| cell.is_some())
                    .map(|cell| cell.as_ref().unwrap().control.as_ref())
                    .filter(|control| control.is_some())
                    .map(|control| control.unwrap().size().1)
                    .fold(0, |s, i| if s > i {s} else {i}),
            layout::Size::MatchParent => base.measured.1 / self.data.cols.len() as u16,
        };
        self.data.cols.iter_mut().for_each(|col| {
            col.control.as_mut().map(|control| {
                control.set_layout_height(layout::Size::Exact(height));
                control.measure(w, h);
                control.draw(None);
            });
        });
        self.data.rows.iter_mut().for_each(|row| {
            row.height = size;
            row.control.as_mut().map(|control| {
                control.set_layout_height(layout::Size::Exact(height));
                control.measure(w, h);
                control.draw(None);
            });
            row.cells.iter_mut().for_each(|cell| {
                cell.as_mut().map(|cell| {
                    cell.control.as_mut().map(|control| {
                        control.set_layout_height(layout::Size::Exact(height));
                        control.measure(w, h);
                        control.draw(None);
                    });
                });
            });
        });
        if self.data.default_row_height != size {
            self.data.row_at_mut(index).map(|row| row.height = size);
        } else {
            let row_height = self.data.default_row_height;
            self.data.row_at_mut(index).map(|mut row| row.height = row_height);
        }
    }
    fn resize_column(&mut self, base: &ControlBase, index: usize, size: layout::Size) {
        let (w, h) = base.measured;
        let mut width = match size {
            layout::Size::Exact(width) => width,
            layout::Size::WrapContent => self.data.rows.iter()
                    .flat_map(|row| row.cells.iter())
                    .filter(|cell| cell.is_some())
                    .map(|cell| cell.as_ref().unwrap().control.as_ref())
                    .filter(|control| control.is_some())
                    .map(|control| control.unwrap().size().0)
                    .fold(0, |s, i| if s > i {s} else {i}),
            layout::Size::MatchParent => w / self.data.cols.len() as u16,
        };
        self.data.column_at_mut(index).map(|col| {
            col.width = size;
            col.control.as_mut().map(|control| {
                control.set_layout_width(layout::Size::Exact(width));
                control.measure(w, h);
                control.draw(None);
            });
        });
        self.data.rows.iter_mut().for_each(|row| {
            row.cell_at_mut(index).map(|cell| {
                cell.control.as_mut().map(|control| {
                    control.set_layout_width(layout::Size::Exact(width));
                    control.measure(w, h);
                    control.draw(None);
                });
            });
        });
    }  
}

impl<O: controls::Table> NewTableInner<O> for CocoaTable {
    fn with_uninit_params(ptr: &mut mem::MaybeUninit<O>, width: usize, height: usize) -> Self {
        let base = common::CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>);
        let base_bounds: NSRect = unsafe { msg_send![base.control, bounds] };
        let table = unsafe {
            let mut control: cocoa_id = msg_send![WINDOW_CLASS_INNER.0, alloc];
            control = msg_send![control, initWithFrame: base_bounds];
            control
        };
        let li = CocoaTable {
            base: base,
            table: table,
            header: unsafe { msg_send![table, headerView] },
            on_item_click: None,
            data: Default::default(),
        };
        let selfptr = ptr as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            let control = li.base.control;
            (&mut *control).set_ivar(common::IVAR, selfptr);
            let table = li.table;
            (&mut *table).set_ivar(common::IVAR, selfptr);
            
            let column: cocoa_id = msg_send![Class::get("NSTableColumn").unwrap(), alloc];
            let ident = NSString::alloc(nil).init_str("_");
            //let column: cocoa_id = msg_send![column, initWithIdentifier:ident];
            //let () = msg_send![table, addTableColumn: column];
            
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
impl TableInner for CocoaTable {
    fn with_adapter_initial_size(adapter: Box<dyn types::Adapter>, width: usize, height: usize) -> Box<dyn controls::Table> {
        let mut b: Box<mem::MaybeUninit<Table>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AContainer::with_inner(
                    AAdapted::with_inner(
                        ATable::with_inner(
                            <Self as NewTableInner<Table>>::with_uninit_params(b.as_mut(), width, height)
                        ),
                        adapter,
                        &mut b,
                    ),
                )
            ),
        );
        let mut bb = unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        };
        let (member, _, adapter, table) = unsafe { Table::adapter_base_parts_mut(&mut bb.base) };
        adapter.adapter.for_each(&mut (|indexes, node| {
            match node {
                adapter::Node::Leaf => table.inner_mut().add_cell_inner(member, indexes[0], indexes[1]),
                adapter::Node::Branch(_) => table.inner_mut().add_column_inner(member, indexes[0])
            }
        }));
        bb
    }
    fn headers_visible(&self, _: &MemberBase, _: &ControlBase, _: &AdaptedBase) -> bool {
        unsafe { 
            let header: cocoa_id = msg_send![self.table, headerView];
            header != nil
        }
    }
    fn set_headers_visible(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &mut AdaptedBase, visible: bool) {
        unsafe { let () = if visible { msg_send![self.table, setHeaderView:self.header] } else { msg_send![self.table, setHeader:nil] }; }
    }
    fn set_column_width(&mut self, _: &mut MemberBase, control: &mut ControlBase, _: &mut AdaptedBase, index: usize, size: layout::Size) {
        self.resize_column(control, index, size)
    }
    fn set_row_height(&mut self, _: &mut MemberBase, control: &mut ControlBase, _: &mut AdaptedBase, index: usize, size: layout::Size) {
        self.resize_row(control, index, size, false)
    }
}
impl ItemClickableInner for CocoaTable {
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
impl AdaptedInner for CocoaTable {
    fn on_item_change(&mut self, base: &mut MemberBase, value: adapter::Change) {
		match value {
            adapter::Change::Added(at, node) => {
                if adapter::Node::Leaf == node || at.len() > 1 {
                    self.add_cell_inner(base, at[0], at[1]);
                } else {
                    self.add_column_inner(base, at[0]);
                }
            },
            adapter::Change::Removed(at) => {
                if at.len() > 1 {
                    self.remove_cell_inner(base, at[0], at[1]);
                } else {
                    self.remove_column_inner(base, at[0]);
                }
            },
            adapter::Change::Edited(at, node) => {
                if adapter::Node::Leaf == node || at.len() > 1 {
                    self.change_cell_inner(base, at[0], at[1]);
                } else {
                    self.change_column_inner(base, at[0]);
                }
            },
        }
        self.base.invalidate();
	}
}

impl ContainerInner for CocoaTable {
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        for column in self.data.cols.as_mut_slice() {
            let maybe = column.control.as_mut().and_then(|control| utils::find_by_mut(control.as_mut(), arg));
            if maybe.is_some() {
                return maybe;
            }
        }
        for row in self.data.rows.as_mut_slice() {
            for cell in row.cells.as_mut_slice() {
                if let Some(cell) = cell {
                    let maybe = cell.control.as_mut().and_then(|control| utils::find_by_mut(control.as_mut(), arg));
                    if maybe.is_some() {
                        return maybe;
                    }
                }
            }
        }
        None
    }
    fn find_control<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
        for column in self.data.cols.as_slice() {
            let maybe = column.control.as_ref().and_then(|control| utils::find_by(control.as_ref(), arg));
            if maybe.is_some() {
                return maybe;
            }
        }
        for row in self.data.rows.as_slice() {
            for cell in row.cells.as_slice() {
                if let Some(cell) = cell {
                    let maybe = cell.control.as_ref().and_then(|control| utils::find_by(control.as_ref(), arg));
                    if maybe.is_some() {
                        return maybe;
                    }
                }
            }
        }
        None
    }
}

impl ControlInner for CocoaTable {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, parent: &dyn controls::Container, x: i32, y: i32, pw: u16, ph: u16) {
        unsafe { (&mut *self.table).set_ivar(common::IVAR_PARENT, parent.native_id() as *mut c_void); }
        control.coords = Some((x, y));
        self.measure(member, control, pw, ph);

        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        self.data.cols.iter_mut().enumerate().for_each(|(index, col)| {
            //col.control.as_mut().map(|control| set_parent(control.as_mut(), Some(&parent)));
            col.control.as_mut().map(|mut control| control.on_added_to_container(this, 0, 0, pw, ph));
            this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().resize_column(control, index, col.width);
        });
        self.data.rows.iter_mut().enumerate().for_each(|(index, row)| {
            this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().resize_row(control, index, row.height, false);
            //row.control.as_mut().map(|control| set_parent(control.as_mut(), Some(this)));
            row.cells.iter_mut()
                .filter(|cell| cell.is_some())
                .for_each(|cell| {
                    cell.as_mut().and_then(|cell| cell.control.as_mut())
                        .map(|control| control.on_added_to_container(this, 0, 0, pw, ph));
                });
        });
        unsafe {                            
            let () = msg_send![self.table, setDelegate: self.table];
            let () = msg_send![self.table, setDataSource: self.table];
        }
    }
    fn on_removed_from_container(&mut self, member: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        unsafe {
            (&mut *self.table).set_ivar(common::IVAR_PARENT, ptr::null_mut::<c_void>());
            let () = msg_send![self.table, setDelegate: nil];
            let () = msg_send![self.table, setDataSource: nil];
        }
        let this: &mut Table = unsafe { utils::base_to_impl_mut(member) };
        self.data.cols.iter_mut().enumerate().for_each(|(_, col)| {
            col.control.as_mut().map(|control| control.on_removed_from_container(this));
        });
        self.data.rows.iter_mut().enumerate().for_each(|(_, row)| {
            row.cells.iter_mut()
                .filter(|cell| cell.is_some())
                .for_each(|cell| {
                    cell.as_mut().unwrap().control.as_mut()
                        .map(|control| control.on_removed_from_container(this));
                });
        });
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
        use plygui_api::markup::MEMBER_TYPE_Table;

        fill_from_markup_base!(self, base, markup, registry, Table, [MEMBER_TYPE_Table]);
        //fill_from_markup_items!(self, base, markup, registry);
    }
}

impl HasLayoutInner for CocoaTable {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl HasNativeIdInner for CocoaTable {
    type Id = common::CocoaId;

    fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaTable {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaTable {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaTable {}

impl Drawable for CocoaTable {
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

impl Drop for CocoaTable {
    fn drop(&mut self) {
        let this: &mut Table = unsafe { common::member_from_cocoa_id_mut(self.base.control).unwrap() };
        self.data.cols.iter_mut().enumerate().for_each(|(_, col)| {
            col.control.as_mut().map(|control| control.on_removed_from_container(this));
        });
        self.data.rows.iter_mut().enumerate().for_each(|(_, row)| {
            row.cells.iter_mut()
                .filter(|cell| cell.is_some())
                .for_each(|cell| {
                    cell.as_mut().unwrap().control.as_mut()
                        .map(|control| control.on_removed_from_container(this));
                });
        });unsafe {
            let () = msg_send![self.table, release];
        }
    }
}

extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = common::member_from_cocoa_id_mut::<Table>(this).unwrap();
        let b2 = common::member_from_cocoa_id_mut::<Table>(this).unwrap();
        (b.inner().inner().inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: controls::Table>(this: &mut Table, _: Sel, param: NSSize) {
    unsafe {
        if let Some(cls) = Class::get("NSScrollView") {
            let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().base.control, cls), setFrameSize: param];
            this.call_on_size::<O>(param.width as u16, param.height as u16);
        }
    }
}
impl Spawnable for CocoaTable {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_adapter(Box::new(types::imp::StringVecAdapter::<crate::imp::Text>::new())).into_control()
    }
}

extern "C" fn datasource_len(this: &mut Object, _: Sel, _: cocoa_id) -> NSInteger {
    unsafe {
        let sp = common::member_from_cocoa_id::<Table>(this).unwrap();
        sp.inner().inner().inner().inner().inner().data.rows.len() as i64
    }
}
extern "C" fn spawn_item(this: &mut Object, _: Sel, _: cocoa_id, column: cocoa_id, row: NSInteger) -> cocoa_id {
    let sp = unsafe { common::member_from_cocoa_id_mut::<Table>(this).unwrap() };
    let sp = sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
    let column = sp.data.cols.iter().enumerate()
        .filter(|(_,col)| col.native == column)
        .map(|(index,_)| index).next().expect("Cannot get a column index");
    sp.data.cell_at(&[column, row as usize]).map(|cell| cell.native).unwrap_or(nil)
}
extern "C" fn get_item_height(this: &mut Object, _: Sel, _: cocoa_id, row: NSInteger) -> f64 {
    use crate::plygui_api::controls::Control;
    let sp = unsafe { common::member_from_cocoa_id_mut::<Table>(this).unwrap() };
    let size = sp.as_control().size().1;
    let sp = sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut();
    let height = sp.data.row_at(row as usize).map(|row| row.height).unwrap_or(sp.data.default_row_height);
    let height = match height {
        layout::Size::Exact(height) => height,
        layout::Size::WrapContent => sp.data.row_at(row as usize)
                .iter()
                .flat_map(|row| row.cells.iter())
                .filter(|cell| cell.is_some())
                .map(|cell| cell.as_ref().unwrap().control.as_ref())
                .filter(|control| control.is_some())
                .map(|control| control.unwrap().size().1)
                .fold(0, |s, i| if s > i {s} else {i}),
        layout::Size::MatchParent => size / sp.data.cols.len() as u16,
    };
    if height > 0 { height as f64 } else { 1. }
}
extern "C" fn item_clicked(this: &mut Object, _: Sel, _: cocoa_id) {
    let sp = unsafe { common::member_from_cocoa_id_mut::<Table>(this).unwrap() };
    let sp2 = unsafe { common::member_from_cocoa_id_mut::<Table>(this).unwrap() };
    let x: NSInteger = unsafe { msg_send![this, clickedColumn] };
    if x < 0 {
        return;
    }
    let y: NSInteger = unsafe { msg_send![this, clickedRow] };
    if y < 0 {
        return;
    }
    let item_view = sp.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().data.cell_at_mut(&[x as usize, y as usize]).unwrap();
    if let Some(ref mut callback) = sp2.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().on_item_click {
        let sp2 = unsafe { common::member_from_cocoa_id_mut::<Table>(this).unwrap() };
        if let Some(clicked) = item_view.control.as_mut() {
            (callback.as_mut())(sp2, &[x as usize, y as usize], clicked.as_mut());
        }
    }
}
extern "C" fn validate_proposed_first_responder(_: &mut Object, _: Sel, responder: cocoa_id, evt: cocoa_id) -> BOOL {
    let evt_type: NSEventType = unsafe { evt.eventType() };
    match evt_type {
        NSEventType::NSLeftMouseUp => unsafe { msg_send![responder, isKindOfClass: class!(NSButton)] },
        _ => NO //unsafe { msg_send![super(this, Class::get(BASE_CLASS).unwrap()), validateProposedFirstResponder:responder forEvent:evt] }
    }
}
