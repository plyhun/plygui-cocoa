use crate::common::{self, *};

use cocoa::appkit::{NSSquareStatusItemLength, NSStatusBar};

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
    icon: image::DynamicImage,
    menu: cocoa_id,
    menu_actions: HashMap<cocoa_id, callbacks::Action>,
    on_close: Option<callbacks::OnClose>,
}

pub type Tray = AMember<ACloseable<ATray<CocoaTray>>>;

impl CocoaTray {
    fn install_image(&mut self) {
        unsafe {
            let thickness: f64 = msg_send![NSStatusBar::systemStatusBar(nil), thickness];
            let i = self.icon.resize(thickness as u32, thickness as u32, image::imageops::FilterType::Lanczos3);
            
        	let img = common::image_to_native(&i);
        	let btn: cocoa_id = msg_send![self.tray, button];
        	let () = msg_send![self.tray, setHighlightMode:YES];
        	let () = msg_send![img, setTemplate:YES];
        	let () = msg_send![btn, setImage:img];
        }
    }
}

impl HasLabelInner for CocoaTray {
    fn label(&self, _: &MemberBase) -> Cow<str> {
        unsafe {
            let title: cocoa_id = msg_send![self.tray, title];
            let title = msg_send![title, UTF8String];
            Cow::Owned(ffi::CString::from_raw(title).into_string().unwrap())
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: Cow<str>) {
        unsafe {
            let label = NSString::alloc(cocoa::base::nil).init_str(&label);
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
        true
    }
    fn on_close(&mut self, callback: Option<callbacks::OnClose>) {
        self.on_close = callback;
    }
    fn application<'a>(&'a self, base: &'a MemberBase) -> &'a dyn controls::Application {
        unsafe { utils::base_to_impl::<Tray>(base) }.inner().application_impl::<crate::application::Application>()
    }
    fn application_mut<'a>(&'a mut self, base: &'a mut MemberBase) -> &'a mut dyn controls::Application {
        unsafe { utils::base_to_impl_mut::<Tray>(base) }.inner_mut().application_impl_mut::<crate::application::Application>()
    }
}

impl HasImageInner for CocoaTray {
    fn image(&self, _base: &MemberBase) -> Cow<image::DynamicImage> {
        Cow::Borrowed(&self.icon)
    }
    fn set_image(&mut self, _base: &mut MemberBase, i: Cow<image::DynamicImage>) {
        self.icon = i.into_owned();
        self.install_image();
    }    
}
impl<O: controls::Tray> NewTrayInner<O> for CocoaTray {
    fn with_uninit_params(u: &mut mem::MaybeUninit<O>, _: &str, icon: image::DynamicImage, menu: types::Menu) -> Self {
        CocoaTray {
            tray: ptr::null_mut(),
            this: u as *mut _ as *mut Tray,
            icon: icon,
            menu_actions: if menu.is_some() { HashMap::new() } else { HashMap::with_capacity(0) },
            menu: nil,
            on_close: None,
        }
    }
}
impl TrayInner for CocoaTray {
    fn with_params<S: AsRef<str>>(app: &mut dyn controls::Application, title: S, icon: image::DynamicImage, menu: types::Menu) -> Box<dyn controls::Tray> {
        let app = app.as_any_mut().downcast_mut::<crate::application::Application>().unwrap();
        
        let mut b: Box<mem::MaybeUninit<Tray>> = Box::new_uninit();
        let ab = AMember::with_inner(
            ACloseable::with_inner(
                ATray::with_inner(
                    <Self as NewTrayInner<Tray>>::with_uninit_params(b.as_mut(), title.as_ref(), icon, types::Menu::None),
    	        ),
                app
            )
        );
        let mut t = unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        };
        let status_bar: cocoa_id = unsafe { NSStatusBar::systemStatusBar(nil) };
        t.inner_mut().inner_mut().inner_mut().tray = unsafe { status_bar.statusItemWithLength_(NSSquareStatusItemLength) };
        
        controls::HasLabel::set_label(t.as_mut(), title.as_ref().into());
        t.inner_mut().inner_mut().inner_mut().install_image();
        
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
                
                let selfptr = t.as_mut() as *mut _ as *mut c_void;
                common::make_menu(nsmenu, menu, &mut t.inner_mut().inner_mut().inner_mut().menu_actions, spawn, selfptr);
                nsmenu
            },
            None => nil,
        };

        unsafe {
            let () = msg_send![t.inner_mut().inner_mut().inner_mut().tray, setMenu: menu];
        }
        t
    }
}

impl HasNativeIdInner for CocoaTray {
    type Id = common::CocoaId;

    fn native_id(&self) -> Self::Id {
        self.tray.into()
    }
}

impl MemberInner for CocoaTray {}

extern "C" fn on_tray_menu_item_select(this: &mut Object, _: Sel, _: cocoa_id) -> BOOL {
    let key = this as cocoa_id;
    let tray = unsafe { common::member_from_cocoa_id_mut::<Tray>(this) }.unwrap();
    let tray2 = unsafe { common::member_from_cocoa_id_mut::<Tray>(this) }.unwrap();
    if let Some(action) = tray.inner_mut().inner_mut().inner_mut().menu_actions.get_mut(&key) {
        if !(action.as_mut())(tray2) {
            return NO;
        }
    }
    YES
}
