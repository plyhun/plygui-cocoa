use super::common::*;

use self::cocoa::appkit::{NSBackingStoreBuffered, NSWindow, NSWindowStyleMask};

const BASE_CLASS: &str = "NSWindow";

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe { register_window_class("PlyguiWindow", BASE_CLASS, |_| {}) };
    static ref DELEGATE: common::RefClass = unsafe { register_delegate() };
}

pub type Window = Member<SingleContainer<::plygui_api::development::Window<CocoaWindow>>>;

#[repr(C)]
pub struct CocoaWindow {
    pub(crate) window: cocoa_id,
    pub(crate) container: cocoa_id,

    child: Option<Box<dyn controls::Control>>,
    on_close: Option<callbacks::Action>,
    skip_callbacks: bool,
}

impl CocoaWindow {
    fn size_inner(&self) -> (u16, u16) {
        unsafe {
            let size = NSWindow::frame(self.window.contentView()).size;
            (size.width as u16, size.height as u16)
        }
    }
    fn redraw(&mut self) {
        let size = self.size_inner();
        if let Some(ref mut child) = self.child {
            child.measure(size.0, size.1);
            child.draw(Some((0, 0)));
        }
    }
}

impl CloseableInner for CocoaWindow {
    fn close(&mut self, skip_callbacks: bool) {
        self.skip_callbacks = skip_callbacks;
        let _ = unsafe { msg_send![self.window, close] };        
    }
    fn on_close(&mut self, callback: Option<callbacks::Action>) {
        self.on_close = callback;
    }
}

impl WindowInner for CocoaWindow {
    fn with_params(title: &str, window_size: types::WindowStartSize, _menu: types::WindowMenu) -> Box<Window> {
        use self::cocoa::appkit::NSView;

        unsafe {
            let rect: NSRect = match window_size {
                types::WindowStartSize::Exact(width, height) => NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(width as f64, height as f64),
                ),
                types::WindowStartSize::Fullscreen => {
                    let screen: cocoa_id = msg_send![class!(NSScreen), mainScreen];
                    msg_send![screen, frame]
                },
            };
            let window: cocoa_id = msg_send![WINDOW_CLASS.0, alloc];
            let window = window.initWithContentRect_styleMask_backing_defer_(
                rect,
                NSWindowStyleMask::NSClosableWindowMask | NSWindowStyleMask::NSResizableWindowMask | NSWindowStyleMask::NSMiniaturizableWindowMask | NSWindowStyleMask::NSTitledWindowMask,
                NSBackingStoreBuffered,
                NO,
            );
            let () = msg_send![window ,cascadeTopLeftFromPoint: NSPoint::new(20., 20.)];
            window.center();
            let title = NSString::alloc(cocoa::base::nil).init_str(title);
            let () = msg_send![window, setTitle: title];
            let () = msg_send![window, makeKeyAndOrderFront: cocoa::base::nil];
            let current_app = cocoa::appkit::NSRunningApplication::currentApplication(cocoa::base::nil);
            let () = msg_send![current_app, activateWithOptions: cocoa::appkit::NSApplicationActivateIgnoringOtherApps];

            let view = NSView::alloc(nil).initWithFrame_(rect);
            let () = msg_send![window, setContentView: view];

            let mut window = Box::new(Member::with_inner(
                SingleContainer::with_inner(::plygui_api::development::Window::with_inner(CocoaWindow { window: window, container: view, child: None, on_close: None, skip_callbacks: false }, ()), ()),
                MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
            ));

            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(common::IVAR, window.as_mut() as *mut _ as *mut ::std::os::raw::c_void);
            (&mut *window.as_inner_mut().as_inner_mut().as_inner_mut().window).set_ivar(common::IVAR, window.as_mut() as *mut _ as *mut ::std::os::raw::c_void);
            let () = msg_send![window.as_inner_mut().as_inner_mut().as_inner_mut().window, setDelegate: delegate];

            window
        }
    }
    fn on_frame(&mut self, cb: callbacks::Frame) {
        let _ = unsafe { member_from_cocoa_id_mut::<Window>(self.window) }.unwrap().as_inner_mut().as_inner_mut().base_mut().sender().send(cb);
    }
    fn on_frame_async_feeder(&mut self) -> callbacks::AsyncFeeder<callbacks::Frame> {
        unsafe { member_from_cocoa_id_mut::<Window>(self.window) }.unwrap().as_inner_mut().as_inner_mut().base_mut().sender().clone().into()
    }
}

impl HasLabelInner for CocoaWindow {
    fn label(&self) -> ::std::borrow::Cow<'_, str> {
        unsafe {
            let title: cocoa_id = msg_send![self.window, title];
            let title = msg_send![title, UTF8String];
            Cow::Owned(ffi::CString::from_raw(title).into_string().unwrap())
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: &str) {
        unsafe {
            let label = NSString::alloc(cocoa::base::nil).init_str(label);
            let () = msg_send![self.window, setTitle: label];
        }
    }
}

impl SingleContainerInner for CocoaWindow {
    fn set_child(&mut self, _: &mut MemberBase, mut child: Option<Box<dyn controls::Control>>) -> Option<Box<dyn controls::Control>> {
        use plygui_api::controls::SingleContainer;

        let mut old = self.child.take();
        if let Some(old) = old.as_mut() {
            let outer_self = unsafe { common::member_from_cocoa_id_mut::<Window>(self.window).unwrap() };
            let outer_self = outer_self.as_single_container_mut().as_container_mut();
            old.on_removed_from_container(outer_self);
        }
        if let Some(new) = child.as_mut() {
            let (w, h) = self.size();
            unsafe {
                let () = msg_send![self.container, addSubview: new.native_id() as cocoa_id];
            }
            let outer_self = unsafe { common::member_from_cocoa_id_mut::<Window>(self.window).unwrap() };
            let outer_self = outer_self.as_single_container_mut().as_container_mut();
            new.on_added_to_container(outer_self, 0, 0, w, h);
            new.draw(Some((0, 0)));
        }
        self.child = child;

        old
    }
    fn child(&self) -> Option<&dyn controls::Control> {
        self.child.as_ref().map(|c| c.as_ref())
    }
    fn child_mut(&mut self) -> Option<&mut dyn controls::Control> {
        //self.child.as_mut().map(|c|c.as_mut()) // WTF ??
        if let Some(child) = self.child.as_mut() {
            Some(child.as_mut())
        } else {
            None
        }
    }
}

impl ContainerInner for CocoaWindow {
    fn find_control_by_id_mut(&mut self, id: ids::Id) -> Option<&mut dyn controls::Control> {
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_by_id_mut(id);
            }
        }
        None
    }
    fn find_control_by_id(&self, id: ids::Id) -> Option<&dyn controls::Control> {
        if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control_by_id(id);
            }
        }
        None
    }
}

impl MemberInner for CocoaWindow {
    type Id = common::CocoaId;

    fn size(&self) -> (u16, u16) {
        self.size_inner()
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        unsafe {
            let () = if types::Visibility::Visible == base.visibility {
                msg_send![self.window, setIsVisible: YES]
            } else {
                msg_send![self.window, setIsVisible: NO]
            };
        }
    }

    unsafe fn native_id(&self) -> Self::Id {
        self.window.into()
    }
}

impl Drop for CocoaWindow {
    fn drop(&mut self) {
        unsafe {
            let () = msg_send![self.container, dealloc];
            let () = msg_send![self.window, dealloc];
        }
    }
}

unsafe fn register_delegate() -> common::RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiWindowDelegate", superclass).unwrap();

    decl.add_method(sel!(windowShouldClose:), window_should_close as extern "C" fn(&mut Object, Sel, cocoa_id) -> BOOL);
    decl.add_method(sel!(windowDidResize:), window_did_resize as extern "C" fn(&mut Object, Sel, cocoa_id));
    decl.add_method(sel!(windowDidChangeScreen:), window_did_change_screen as extern "C" fn(&mut Object, Sel, cocoa_id));
    //decl.add_method(sel!(windowWillClose:), window_will_close as extern "C" fn(&Object, Sel, id));

    //decl.add_method(sel!(windowDidBecomeKey:), window_did_become_key as extern "C" fn(&Object, Sel, id));
    //decl.add_method(sel!(windowDidResignKey:), window_did_resign_key as extern "C" fn(&Object, Sel, id));

    decl.add_ivar::<*mut c_void>(common::IVAR);
    //decl.add_ivar::<*mut c_void>("plyguiApplication");

    common::RefClass(decl.register())
}

fn window_redraw(this: &mut Object) {
    use plygui_api::controls::Member;

    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    let size = window.size();
    
    window.as_inner_mut().as_inner_mut().as_inner_mut().redraw();
    window.call_on_resize(size.0, size.1);
}

extern "C" fn window_did_resize(this: &mut Object, _: Sel, _: cocoa_id) {
    window_redraw(this)
}

extern "C" fn window_did_change_screen(this: &mut Object, _: Sel, _: cocoa_id) {
    window_redraw(this)
}
extern "C" fn window_should_close(_: &mut Object, _: Sel, param: cocoa_id) -> BOOL {
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(param) }.unwrap();
    if !window.as_inner_mut().as_inner_mut().as_inner_mut().skip_callbacks {
        if let Some(ref mut on_close) = window.as_inner_mut().as_inner_mut().as_inner_mut().on_close {
            let window2 = unsafe { common::member_from_cocoa_id_mut::<Window>(param) }.unwrap();
            if !(on_close.as_mut())(window2) {
                return NO;
            }
        }
    }
    YES
}

impl_all_defaults!(Window);
