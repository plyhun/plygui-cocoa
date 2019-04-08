use crate::common::{self, *};

use self::cocoa::appkit::{NSSquareStatusItemLength, NSStatusBar};

lazy_static! {
    static ref PLYGUI_MENU_ITEM_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiTrayMenuItem", "NSMenuItem", |decl| {
            decl.add_method(sel!(onTrayMenuItemSelect:), on_tray_menu_item_select as extern "C" fn(&mut Object, Sel, cocoa_id) -> BOOL);
        })
    };
}

#[repr(C)]
pub struct CocoaTray {
    tray: cocoa_id,
    this: *mut Tray,
    menu: cocoa_id,
    menu_actions: HashMap<cocoa_id, callbacks::Action>,
    on_close: Option<callbacks::Action>,
}

pub type Tray = Member<CocoaTray>;

impl HasLabelInner for CocoaTray {
    fn label(&self) -> ::std::borrow::Cow<'_, str> {
        unsafe {
            let title: cocoa_id = msg_send![self.tray, title];
            let title = msg_send![title, UTF8String];
            Cow::Owned(ffi::CString::from_raw(title).into_string().unwrap())
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: &str) {
        unsafe {
            let label = NSString::alloc(cocoa::base::nil).init_str(label);
            let () = msg_send![self.tray, setTitle: label];
        }
    }
}

impl CloseableInner for CocoaTray {
    fn close(&mut self, skip_callbacks: bool) -> bool {
        if !skip_callbacks {
            if let Some(ref mut on_close) = self.on_close {
                if !(on_close.as_mut())(unsafe { &mut *self.this }) {
                    return false;
                }
            }
        }
        unsafe {
            let status_bar: cocoa_id = NSStatusBar::systemStatusBar(ptr::null_mut());
            status_bar.removeStatusItem_(self.tray);
        }
        let mut app = super::application::Application::get();
        app.as_any_mut().downcast_mut::<super::application::Application>().unwrap().as_inner_mut().remove_tray(self.tray.into());
        true
    }
    fn on_close(&mut self, callback: Option<callbacks::Action>) {
        self.on_close = callback;
    }
}

impl TrayInner for CocoaTray {
    fn with_params(title: &str, menu: types::Menu) -> Box<Member<Self>> {
        use plygui_api::controls::HasLabel;

        let status_bar: cocoa_id = unsafe { NSStatusBar::systemStatusBar(nil) };

        let mut t = Box::new(Member::with_inner(
            CocoaTray {
                tray: unsafe { status_bar.statusItemWithLength_(NSSquareStatusItemLength) },
                this: ptr::null_mut(),
                menu_actions: if menu.is_some() { HashMap::new() } else { HashMap::with_capacity(0) },
                menu: nil,
                on_close: None,
            },
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));

        let selfptr = t.as_mut() as *mut Tray;
        t.set_label(title);
        t.as_inner_mut().this = selfptr;

        let menu = match menu {
            Some(menu) => unsafe {
                let nsmenu = NSMenu::new(nil);
                //let () = msg_send![nsmenu, setTitle: title];

                unsafe fn spawn(title: cocoa_id, selfptr: *mut c_void) -> cocoa_id {
                    let item: cocoa_id = msg_send![PLYGUI_MENU_ITEM_CLASS.0, alloc];
                    let item: cocoa_id = msg_send![item, initWithTitle:title action:sel!(onTrayMenuItemSelect:) keyEquivalent:NSString::alloc(nil).init_str("")];
                    let () = msg_send![item, setTarget: item];
                    (&mut *item).set_ivar(IVAR, selfptr);
                    item
                }

                common::make_menu(nsmenu, menu, &mut t.as_inner_mut().menu_actions, spawn, selfptr as *mut c_void);
                nsmenu
            },
            None => nil,
        };

        unsafe {
            let () = msg_send![t.as_inner_mut().tray, setMenu: menu];
        }
        t
    }
}

impl HasNativeIdInner for CocoaTray {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.tray.into()
    }
}

impl MemberInner for CocoaTray {}

impl Drop for CocoaTray {
    fn drop(&mut self) {
        self.close(true);
        unsafe {
            if !self.menu.is_null() {
                let () = msg_send![self.menu, dealloc];
            }
            let () = msg_send![self.tray, dealloc];
        }
    }
}

extern "C" fn on_tray_menu_item_select(this: &mut Object, _: Sel, _: cocoa_id) -> BOOL {
    let key = this as cocoa_id;
    let tray = unsafe { common::member_from_cocoa_id_mut::<Tray>(this) }.unwrap();
    let tray2 = unsafe { common::member_from_cocoa_id_mut::<Tray>(this) }.unwrap();
    if let Some(action) = tray.as_inner_mut().menu_actions.get_mut(&key) {
        if !(action.as_mut())(tray2) {
            return NO;
        }
    }
    YES
}

default_impls_as!(Tray);
