pub use super::*;

pub use std::borrow::Cow;
pub use std::os::raw::c_void;
pub use std::{any, cmp, ffi, marker, mem, ptr, slice, str, sync::mpsc};

pub use self::cocoa::appkit::NSView;
pub use self::cocoa::base::{id as cocoa_id, nil};
pub use self::cocoa::foundation::{NSPoint, NSRange, NSRect, NSSize, NSString};
pub use objc::declare::ClassDecl;
pub use objc::runtime::{class_copyIvarList, Class, Ivar, Object, Sel, BOOL, NO, YES};

pub use plygui_api::development::*;
pub use plygui_api::{callbacks, controls, defaults, ids, layout, types, utils};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RefClass(pub *const Class);
unsafe impl Sync for RefClass {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CocoaId(cocoa_id);

impl From<cocoa_id> for CocoaId {
    fn from(a: cocoa_id) -> CocoaId {
        CocoaId(a)
    }
}
impl From<CocoaId> for cocoa_id {
    fn from(a: CocoaId) -> cocoa_id {
        a.0
    }
}
impl From<CocoaId> for usize {
    fn from(a: CocoaId) -> usize {
        a.0 as usize
    }
}
impl NativeId for CocoaId {}

pub const IVAR: &str = "plyguiIvar";
pub const IVAR_PARENT: &str = "plyguiIvarParent";

#[repr(C)]
pub struct CocoaControlBase<T: controls::Control + Sized + 'static> {
    pub control: cocoa_id,
    pub coords: Option<(i32, i32)>,
    pub measured_size: (u16, u16),
    _marker: marker::PhantomData<T>,
}

impl<T: controls::Control + Sized> CocoaControlBase<T> {
    pub fn with_params(class: RefClass) -> CocoaControlBase<T> {
        CocoaControlBase {
            control: unsafe {
                let rect = NSRect::new(NSPoint::new(0f64, 0f64), NSSize::new(0f64, 0f64));
                let mut control: cocoa_id = msg_send![class.0, alloc];
                control = msg_send![control, initWithFrame: rect];
                control
            },
            coords: None,
            measured_size: (0, 0),

            _marker: marker::PhantomData,
        }
    }
    pub fn size(&self) -> (u16, u16) {
        let frame = self.frame();
        if frame.size.width < 1.0 && frame.size.height < 1.0 {
            self.measured_size
        } else {
            (frame.size.width as u16, frame.size.height as u16)
        }
    }
    pub fn frame(&self) -> NSRect {
        unsafe { msg_send![self.control, frame] }
    }
    pub unsafe fn on_removed_from_container(&mut self) {
        self.control.removeFromSuperview();
    }
    pub fn parent_cocoa_id(&self) -> Option<cocoa_id> {
        unsafe { parent_cocoa_id(self.control, false) }
    }
    pub fn parent(&self) -> Option<&controls::Member> {
        unsafe { parent_cocoa_id(self.control, false).and_then(|id| member_base_from_cocoa_id(id).map(|m| m.as_member())) }
    }
    pub fn parent_mut(&mut self) -> Option<&mut controls::Member> {
        unsafe { parent_cocoa_id(self.control, false).and_then(|id| member_base_from_cocoa_id_mut(id).map(|m| m.as_member_mut())) }
    }
    pub fn root(&self) -> Option<&controls::Member> {
        unsafe { parent_cocoa_id(self.control, true).and_then(|id| member_base_from_cocoa_id(id).map(|m| m.as_member())) }
    }
    pub fn root_mut(&mut self) -> Option<&mut controls::Member> {
        unsafe { parent_cocoa_id(self.control, true).and_then(|id| member_base_from_cocoa_id_mut(id).map(|m| m.as_member_mut())) }
    }
    pub fn on_set_visibility(&mut self, base: &mut MemberBase) {
        unsafe {
            let () = if types::Visibility::Visible == base.visibility {
                msg_send![self.control, setHidden: NO]
            } else {
                msg_send![self.control, setHidden: YES]
            };
        }
        self.invalidate();
    }
    pub fn draw(&mut self, coords: Option<(i32, i32)>) {
        if coords.is_some() {
            self.coords = coords;
        }
        if let Some((x, y)) = self.coords {
            let (_, ph) = self.parent().unwrap().size();
            unsafe {
                let mut frame: NSRect = self.frame();
                frame.size = NSSize::new((self.measured_size.0 as i32) as f64, (self.measured_size.1 as i32) as f64);
                frame.origin = NSPoint::new(x as f64, (ph as i32 - y - self.measured_size.1 as i32) as f64);
                let () = msg_send![self.control, setFrame: frame];
            }
        }
    }
    pub fn invalidate(&mut self) {
        let parent_id = self.parent_cocoa_id();
        if let Some(parent_id) = parent_id {
            let mparent = unsafe { member_base_from_cocoa_id_mut(parent_id).unwrap().as_member_mut() };
            let (pw, ph) = mparent.size();
            let this = unsafe { member_from_cocoa_id_mut::<T>(self.control).unwrap() };

            let (_, _, changed) = this.measure(pw, ph);

            if changed {
                let mparent_type = mparent.as_any().get_type_id();
                if let Some(control) = mparent.is_control_mut() {
                    control.invalidate();
                } else if mparent_type == any::TypeId::of::<super::window::Window>() {
                    this.draw(None);
                    unsafe {
                        let () = msg_send![parent_id, setNeedsDisplay: YES];
                    }
                } else {
                    panic!("Parent member is unsupported, neither a control, nor a window");
                }
            } else {
                this.draw(None);
                unsafe {
                    let () = msg_send![parent_id, setNeedsDisplay: YES];
                }
            }
        }
    }
    pub fn as_outer(&self) -> &T {
        unsafe { common::member_from_cocoa_id(self.control).unwrap() }
    }
    pub fn as_outer_mut(&mut self) -> &mut T {
        unsafe { common::member_from_cocoa_id_mut(self.control).unwrap() }
    }
}

impl<T: controls::Control + Sized> Drop for CocoaControlBase<T> {
    fn drop(&mut self) {
        unsafe {
            self.on_removed_from_container();
            let () = msg_send![self.control, release];
            //let () = msg_send![self.control, dealloc];
        }
    }
}

pub unsafe fn parent_cocoa_id(id: cocoa_id, is_root: bool) -> Option<cocoa_id> {
    let id_: cocoa_id = if is_root {
        msg_send![id, window]
    } else if let Some(parent) = has_cocoa_id_ivar(id, IVAR_PARENT) {
        parent as cocoa_id
    } else {
        msg_send![id, superview]
    };
    if id_.is_null() || id_ == id {
        None
    } else {
        Some(id_)
    }
}
pub unsafe fn member_base_from_cocoa_id_mut<'a>(id: cocoa_id) -> Option<&'a mut MemberBase> {
    cast_cocoa_id_to_ptr(id).map(|ptr| mem::transmute(ptr as *mut _ as *mut ::std::os::raw::c_void))
}
pub unsafe fn member_base_from_cocoa_id<'a>(id: cocoa_id) -> Option<&'a MemberBase> {
    cast_cocoa_id_to_ptr(id).map(|ptr| mem::transmute(ptr as *mut _ as *const ::std::os::raw::c_void))
}
pub unsafe fn member_from_cocoa_id_mut<'a, T>(id: cocoa_id) -> Option<&'a mut T>
where
    T: controls::Member + Sized,
{
    cast_cocoa_id_to_ptr(id).map(|ptr| mem::transmute(ptr as *mut _ as *mut ::std::os::raw::c_void))
}
pub unsafe fn member_from_cocoa_id<'a, T>(id: cocoa_id) -> Option<&'a T>
where
    T: controls::Member + Sized,
{
    cast_cocoa_id_to_ptr(id).map(|ptr| mem::transmute(ptr as *mut _ as *const ::std::os::raw::c_void))
}
pub unsafe fn cast_cocoa_id_to_ptr<'a>(id: cocoa_id) -> Option<*mut c_void> {
    if id.is_null() {
        return None;
    }

    if let Some(parent) = has_cocoa_id_ivar(id, IVAR) {
        Some(parent)
    } else {
        parent_cocoa_id(id, true).and_then(|id| cast_cocoa_id_to_ptr(id))
    }
}

pub unsafe fn has_cocoa_id_ivar(id: cocoa_id, ivar: &str) -> Option<*mut c_void> {
    if id.is_null() {
        return None;
    }
    let mut ivar_count = 0;
    let class = msg_send![id, class];
    let ivars = class_copyIvarList(class, &mut ivar_count);
    let ivars: &[&Ivar] = slice::from_raw_parts_mut(ivars as *mut _, ivar_count as usize);

    if ivars.iter().any(|va| va.name() == ivar) {
        let ivar: *mut c_void = *(&mut *id).get_ivar(ivar);
        if !ivar.is_null() {
            return Some(ivar);
        }
    }
    None
}

pub unsafe fn measure_string(text: &str) -> (u16, u16) {
    let title = NSString::alloc(cocoa::base::nil).init_str(text);
    measure_nsstring(title)
}

pub unsafe fn measure_nsstring(title: cocoa_id) -> (u16, u16) {
    let text_storage: cocoa_id = msg_send![class!(NSTextStorage), alloc];
    let text_storage: cocoa_id = msg_send![text_storage, initWithString: title];
    let layout_manager: cocoa_id = msg_send![class!(NSLayoutManager), alloc];
    let layout_manager: cocoa_id = msg_send![layout_manager, init];
    let text_container: cocoa_id = msg_send![class!(NSTextContainer), alloc];
    let text_container: cocoa_id = msg_send![text_container, init];

    let () = msg_send![layout_manager, addTextContainer: text_container];
    let () = msg_send![text_container, release];
    let () = msg_send![text_storage, addLayoutManager: layout_manager];
    let () = msg_send![layout_manager, release];

    let num = msg_send![layout_manager, numberOfGlyphs];
    let range = NSRange::new(0, num);

    let string_rect: NSRect = msg_send![layout_manager, boundingRectForGlyphRange:range inTextContainer:text_container];
    (string_rect.size.width as u16, string_rect.size.height as u16)
}

pub unsafe fn register_window_class<F>(name: &str, base: &str, mut f: F) -> RefClass
where
    F: FnMut(&mut ClassDecl),
{
    let superclass = Class::get(base).unwrap();
    let mut decl = ClassDecl::new(name, superclass).unwrap();

    decl.add_ivar::<*mut c_void>(IVAR);
    decl.add_ivar::<*mut c_void>(IVAR_PARENT);

    f(&mut decl);

    common::RefClass(decl.register())
}
