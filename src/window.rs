use crate::common::{self, *};

use cocoa::appkit::{NSBackingStoreBuffered, NSView, NSWindow, NSWindowStyleMask};

const BASE_CLASS: &str = "NSWindow";

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe { register_window_class("PlyguiWindow", BASE_CLASS, |_| {}) };
    static ref DELEGATE: common::RefClass = unsafe { register_delegate() };
    static ref PLYGUI_MENU_ITEM_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiWindowMenuItem", "NSMenuItem", |decl| {
            decl.add_method(sel!(onWindowMenuItemSelect:), on_window_menu_item_select as extern "C" fn(&mut Object, Sel, cocoa_id) -> BOOL);
        })
    };
}

pub type Window = AMember<AContainer<ASingleContainer<AWindow<CocoaWindow>>>>;

#[repr(C)]
pub struct CocoaWindow {
    pub(crate) window: cocoa_id,
    pub(crate) container: cocoa_id,
    menu: cocoa_id,

    child: Option<Box<dyn controls::Control>>,
    menu_actions: HashMap<cocoa_id, callbacks::Action>,
    on_close: Option<callbacks::OnClose>,
    skip_callbacks: bool,
    closed: bool,
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
    fn close(&mut self, skip_callbacks: bool) -> bool {
        self.skip_callbacks = skip_callbacks;
        let () = unsafe { msg_send![self.window, performClose:self.window] };
        let visible: BOOL = unsafe { msg_send![self.window, isVisible] };
        if visible == NO {
            let mut app = super::application::Application::get().unwrap();
            app.as_any_mut().downcast_mut::<super::application::Application>().unwrap().inner_mut().remove_window(self.window.into());
            true
        } else {
            false
        }
    }
    fn on_close(&mut self, callback: Option<callbacks::OnClose>) {
        self.on_close = callback;
    }
}

impl WindowInner for CocoaWindow {
    fn with_params<S: AsRef<str>>(title: S, window_size: types::WindowStartSize, menu: types::Menu) -> Box<dyn controls::Window> {
        unsafe {
            let rect: NSRect = match window_size {
                types::WindowStartSize::Exact(width, height) => NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width as f64, height as f64)),
                types::WindowStartSize::Fullscreen => {
                    let screen: cocoa_id = msg_send![class!(NSScreen), mainScreen];
                    msg_send![screen, frame]
                }
            };
            let title = NSString::alloc(cocoa::base::nil).init_str(title.as_ref());
            let window: cocoa_id = msg_send![WINDOW_CLASS.0, alloc];
            let window = window.initWithContentRect_styleMask_backing_defer_(
                rect,
                NSWindowStyleMask::NSClosableWindowMask | NSWindowStyleMask::NSResizableWindowMask | NSWindowStyleMask::NSMiniaturizableWindowMask | NSWindowStyleMask::NSTitledWindowMask,
                NSBackingStoreBuffered,
                NO,
            );
            let () = msg_send![window ,cascadeTopLeftFromPoint: NSPoint::new(20., 20.)];
            window.center();
            let () = msg_send![window, setTitle: title];
            let () = msg_send![window, makeKeyAndOrderFront: nil];
            let current_app = cocoa::appkit::NSRunningApplication::currentApplication(nil);
            let () = msg_send![current_app, activateWithOptions: cocoa::appkit::NSApplicationActivateIgnoringOtherApps];

            let view = NSView::alloc(nil).initWithFrame_(rect);
            let () = msg_send![window, setContentView: view];

            let mut window = Box::new(AMember::with_inner(
                AContainer::with_inner(
                    ASingleContainer::with_inner(
                        AWindow::with_inner(
                            CocoaWindow {
                                window: window,
                                container: view,
                                menu: nil,
                                child: None,
                                menu_actions: if menu.is_some() { HashMap::new() } else { HashMap::with_capacity(0) },
                                on_close: None,
                                skip_callbacks: false,
                                closed: false,
                            },
                        ),
                    ),
                )
            ));

            let selfptr = window.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(common::IVAR, selfptr);
            (&mut *window.inner_mut().inner_mut().inner_mut().inner_mut().window).set_ivar(common::IVAR, selfptr);
            let () = msg_send![window.inner_mut().inner_mut().inner_mut().inner_mut().window, setDelegate: delegate];
            let () = msg_send![window.inner_mut().inner_mut().inner_mut().inner_mut().window, makeKeyAndOrderFront: nil];

            window.inner_mut().inner_mut().inner_mut().inner_mut().menu = match menu {
                Some(menu) => {
                    let nsmenu = NSMenu::new(view);
                    let () = msg_send![nsmenu, setTitle: title];

                    unsafe fn spawn(title: cocoa_id, selfptr: *mut c_void) -> cocoa_id {
                        let item: cocoa_id = msg_send![PLYGUI_MENU_ITEM_CLASS.0, alloc];
                        let item: cocoa_id = msg_send![item, initWithTitle:title action:sel!(onWindowMenuItemSelect:) keyEquivalent:NSString::alloc(cocoa::base::nil).init_str("")];
                        let () = msg_send![item, setTarget: item];
                        (&mut *item).set_ivar(IVAR, selfptr);
                        item
                    }

                    common::make_menu(nsmenu, menu, &mut window.inner_mut().inner_mut().inner_mut().inner_mut().menu_actions, spawn, selfptr);
                    nsmenu
                }
                None => nil,
            };

            window
        }
    }
    fn size(&self) -> (u16, u16) {
        self.size_inner()
    }
    fn position(&self) -> (i32, i32) {
        unsafe {
            let frame: NSRect = msg_send![self.window, frame];
            (frame.origin.x as i32, frame.origin.y as i32 - frame.size.height as i32)
        }
    }
}

impl HasLabelInner for CocoaWindow {
    fn label(&self, _base: &MemberBase) -> ::std::borrow::Cow<'_, str> {
        unsafe {
            let title: cocoa_id = msg_send![self.window, title];
            let title = msg_send![title, UTF8String];
            Cow::Owned(ffi::CString::from_raw(title).into_string().unwrap())
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: Cow<str>) {
        unsafe {
            let label = NSString::alloc(cocoa::base::nil).init_str(&label);
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
    fn find_control_mut(&mut self, arg: types::FindBy) -> Option<&mut dyn controls::Control> {
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_mut(arg);
            }
        }
        None
    }
    fn find_control(&self, arg: types::FindBy) -> Option<&dyn controls::Control> {
        if let Some(child) = self.child.as_ref() {
            if let Some(c) = child.is_container() {
                return c.find_control(arg);
            }
        }
        None
    }
}

impl HasNativeIdInner for CocoaWindow {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.window.into()
    }
}

impl HasSizeInner for CocoaWindow {
    fn on_size_set(&mut self, _base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        unsafe {
            let mut frame: NSRect = msg_send![self.window, frame];
            frame.size = NSSize::new(width as f64, height as f64);
            let () = msg_send![self.window, setFrame: frame];
        }
        true
    }
}

impl HasVisibilityInner for CocoaWindow {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        unsafe {
            let () = if types::Visibility::Visible == value {
                msg_send![self.window, setIsVisible: YES]
            } else {
                msg_send![self.window, setIsVisible: NO]
            };
        }
        true
    }
}

impl MemberInner for CocoaWindow {}

impl Drop for CocoaWindow {
    fn drop(&mut self) {
        unsafe {
            let () = msg_send![self.container, dealloc];
            let () = msg_send![self.window, dealloc];
            if !self.menu.is_null() {
                let () = msg_send![self.menu, dealloc];
            }
            for (k,_) in self.menu_actions.drain() {
                let () = msg_send![k, dealloc];
            }
        }
        self.close(true);
    }
}

unsafe fn register_delegate() -> common::RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiWindowDelegate", superclass).unwrap();

    decl.add_method(sel!(windowShouldClose:), window_should_close as extern "C" fn(&mut Object, Sel, cocoa_id) -> BOOL);
    decl.add_method(sel!(windowDidResize:), window_did_resize as extern "C" fn(&mut Object, Sel, cocoa_id));
    decl.add_method(sel!(windowDidChangeScreen:), window_did_change_screen as extern "C" fn(&mut Object, Sel, cocoa_id));
    decl.add_method(sel!(windowDidBecomeKey:), window_did_become_key as extern "C" fn(&mut Object, Sel, cocoa_id));

    decl.add_ivar::<*mut c_void>(common::IVAR);

    common::RefClass(decl.register())
}

fn window_redraw(this: &mut Object) {
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    let size = controls::HasSize::size(window);

    window.inner_mut().inner_mut().inner_mut().inner_mut().redraw();
    window.call_on_size(size.0, size.1);
}

extern "C" fn window_did_become_key(this: &mut Object, _: Sel, _: cocoa_id) {
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    let menu = window.inner_mut().inner_mut().inner_mut().inner_mut().menu;
    //if !menu.is_null() {
    let mut app = super::application::Application::get().unwrap();
    app.as_any_mut().downcast_mut::<super::application::Application>().unwrap().inner_mut().set_app_menu(menu);
    //}
}

extern "C" fn window_did_resize(this: &mut Object, _: Sel, _: cocoa_id) {
    window_redraw(this)
}

extern "C" fn window_did_change_screen(this: &mut Object, _: Sel, _: cocoa_id) {
    window_redraw(this)
}
extern "C" fn window_should_close(_: &mut Object, _: Sel, param: cocoa_id) -> BOOL {
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(param) }.unwrap();
    if !window.inner_mut().inner_mut().inner_mut().inner_mut().skip_callbacks {
        if let Some(ref mut on_close) = window.inner_mut().inner_mut().inner_mut().inner_mut().on_close {
            let window2 = unsafe { common::member_from_cocoa_id_mut::<Window>(param) }.unwrap();
            if !(on_close.as_mut())(window2) {
                return NO;
            }
        }
    }
    let mut app = super::application::Application::get().unwrap();
    app.as_any_mut().downcast_mut::<super::application::Application>().unwrap().inner_mut().remove_window(param.into());
    YES
}

extern "C" fn on_window_menu_item_select(this: &mut Object, _: Sel, _: cocoa_id) -> BOOL {
    let key = this as cocoa_id;
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    let window2 = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    if let Some(action) = window.inner_mut().inner_mut().inner_mut().inner_mut().menu_actions.get_mut(&key) {
        if !(action.as_mut())(window2) {
            return NO;
        }
    }
    YES
}
