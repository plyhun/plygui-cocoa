use crate::common::{self, *};

use cocoa::appkit::{NSBackingStoreBuffered, NSView, NSWindow, NSWindowStyleMask};
use dispatch::Queue;

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

pub type Window = AMember<AContainer<ASingleContainer<ACloseable<AWindow<CocoaWindow>>>>>;

#[repr(C)]
pub struct CocoaWindow {
    pub(crate) window: cocoa_id,
    pub(crate) container: cocoa_id,
    menu: cocoa_id,
    
    resize: fn(this: &mut Window),

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
        visible == NO
    }
    fn on_close(&mut self, callback: Option<callbacks::OnClose>) {
        self.on_close = callback;
    }
    fn application<'a>(&'a self, base: &'a MemberBase) -> &'a dyn controls::Application {
        unsafe { utils::base_to_impl::<Window>(base) }.inner().inner().inner().application_impl::<crate::application::Application>()
    }
    fn application_mut<'a>(&'a mut self, base: &'a mut MemberBase) -> &'a mut dyn controls::Application {
        unsafe { utils::base_to_impl_mut::<Window>(base) }.inner_mut().inner_mut().inner_mut().application_impl_mut::<crate::application::Application>()
    }
}

impl<O: controls::Window> NewWindowInner<O> for CocoaWindow {
    fn with_uninit_params(u: &mut mem::MaybeUninit<O>, _: &mut dyn controls::Application, title: &str, window_size: types::WindowStartSize, menu: types::Menu) -> Self {
        let selfptr = u as *mut _ as *mut c_void;
   		let rect: NSRect = match window_size {
            types::WindowStartSize::Exact(width, height) => NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width as f64, height as f64)),
            types::WindowStartSize::Fullscreen => {
                let screen: cocoa_id = unsafe { msg_send![class!(NSScreen), mainScreen] };
                unsafe { msg_send![screen, frame] }
            }
        };
   		let mut w = unsafe {
            let title = NSString::alloc(cocoa::base::nil).init_str(title);
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
            
            let delegate: *mut Object = msg_send!(DELEGATE.0, new);
            (&mut *delegate).set_ivar(common::IVAR, selfptr);
            (&mut *window).set_ivar(common::IVAR, selfptr);
            let () = msg_send![window, setDelegate: delegate];
            let () = msg_send![window, makeKeyAndOrderFront: nil];
    
            CocoaWindow {
                window: window,
                container: view,
                resize: window_did_change_screen_resize_inner::<O>,
                menu: nil,
                child: None,
                menu_actions: if menu.is_some() { HashMap::new() } else { HashMap::with_capacity(0) },
                on_close: None,
                skip_callbacks: false,
                closed: false,
            }
        };
		w.menu = match menu {
            Some(menu) => unsafe {
                let nsmenu = NSMenu::new(w.container);
                let title = NSString::alloc(cocoa::base::nil).init_str(title);
                let () = msg_send![nsmenu, setTitle: title];

                unsafe fn spawn(title: cocoa_id, selfptr: *mut c_void) -> cocoa_id {
                    let item: cocoa_id = msg_send![PLYGUI_MENU_ITEM_CLASS.0, alloc];
                    let item: cocoa_id = msg_send![item, initWithTitle:title action:sel!(onWindowMenuItemSelect:) keyEquivalent:NSString::alloc(cocoa::base::nil).init_str("")];
                    let () = msg_send![item, setTarget: item];
                    (&mut *item).set_ivar(IVAR, selfptr);
                    item
                }

                common::make_menu(nsmenu, menu, &mut w.menu_actions, spawn, selfptr);
                nsmenu
            }
            None => nil,
        };
		w
    }
}
impl WindowInner for CocoaWindow {
    fn with_params<S: AsRef<str>>(app: &mut dyn controls::Application, title: S, window_size: types::WindowStartSize, menu: types::Menu) -> Box<dyn controls::Window> {
        let app = app.as_any_mut().downcast_mut::<crate::application::Application>().unwrap();
        let mut b: Box<mem::MaybeUninit<Window>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AContainer::with_inner(
                ASingleContainer::with_inner(
                    ACloseable::with_inner(
                        AWindow::with_inner(
                            <Self as NewWindowInner<Window>>::with_uninit_params(b.as_mut(), app, title.as_ref(), window_size, menu),
    	                ),
                        app
                    )
                ),
            )
        );
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
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
    fn find_control_mut<'a>(&'a mut self, arg: types::FindBy<'a>) -> Option<&'a mut dyn controls::Control> {
        if let Some(child) = self.child.as_mut() {
            if let Some(c) = child.is_container_mut() {
                return c.find_control_mut(arg);
            }
        }
        None
    }
    fn find_control<'a>(&'a self, arg: types::FindBy<'a>) -> Option<&'a dyn controls::Control> {
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

    fn native_id(&self) -> Self::Id {
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
            self.container.removeFromSuperview();
        }
    }
}

unsafe fn register_delegate() -> common::RefClass {
    let superclass = Class::get("NSObject").unwrap();
    let mut decl = ClassDecl::new("PlyguiWindowDelegate", superclass).unwrap();

    decl.add_method(sel!(windowShouldClose:), window_should_close as extern "C" fn(&mut Object, Sel, cocoa_id) -> BOOL);
    decl.add_method(sel!(windowDidResize:), window_did_change_screen_resize as extern "C" fn(&mut Object, Sel, cocoa_id));
    decl.add_method(sel!(windowDidChangeScreen:), window_did_change_screen_resize as extern "C" fn(&mut Object, Sel, cocoa_id));
    decl.add_method(sel!(windowDidBecomeKey:), window_did_become_key as extern "C" fn(&mut Object, Sel, cocoa_id));

    decl.add_ivar::<*mut c_void>(common::IVAR);

    common::RefClass(decl.register())
}

fn window_redraw<O: controls::Window>(window: &mut Window) {
    let size = controls::HasSize::size(window);

    window.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().redraw();
    window.call_on_size::<O>(size.0, size.1);
}

extern "C" fn window_did_become_key(this: &mut Object, _: Sel, _: cocoa_id) {
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    let menu = window.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().menu;
    //if !menu.is_null() {
    window.inner_mut().inner_mut().inner_mut().application_impl_mut::<super::application::Application>().inner_mut().set_app_menu(menu);
    //}
}

extern "C" fn window_did_change_screen_resize(this: &mut Object, _: Sel, _: cocoa_id) {
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    let this = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    (window.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().resize)(this)
}
fn window_did_change_screen_resize_inner<O: controls::Window>(this: &mut Window) {
    window_redraw::<O>(this)
}

extern "C" fn window_should_close(_: &mut Object, _: Sel, param: cocoa_id) -> BOOL {
    use crate::plygui_api::controls::Member;
    
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(param) }.unwrap();
    if !window.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().skip_callbacks {
        if let Some(ref mut on_close) = window.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().on_close {
            let window2 = unsafe { common::member_from_cocoa_id_mut::<Window>(param) }.unwrap();
            if !(on_close.as_mut())(window2) {
                return NO;
            }
        }
    }
    let id = window.id();
    let cid = param as usize;
    Queue::main().exec_async(move || {
            let window = unsafe { common::member_from_cocoa_id_mut::<Window>(cid as cocoa_id) }.unwrap();
            let app = window.inner_mut().inner_mut().inner_mut().application_impl_mut::<crate::application::Application>();
            app.base.windows.retain(|w| w.id() != id);
            app.inner_mut().maybe_exit();
    });
    YES
}

extern "C" fn on_window_menu_item_select(this: &mut Object, _: Sel, _: cocoa_id) -> BOOL {
    let key = this as cocoa_id;
    let window = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    let window2 = unsafe { common::member_from_cocoa_id_mut::<Window>(this) }.unwrap();
    if let Some(action) = window.inner_mut().inner_mut().inner_mut().inner_mut().inner_mut().menu_actions.get_mut(&key) {
        if !(action.as_mut())(window2) {
            return NO;
        }
    }
    YES
}
