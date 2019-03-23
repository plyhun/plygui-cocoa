use super::common::*;

use self::cocoa::appkit::{NSSquareStatusItemLength, NSStatusBar, NSStatusItem};

#[repr(C)]
pub struct CocoaTray {
    tray: cocoa_id,
    this: *mut Tray,

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
        app.as_any_mut().downcast_mut::<super::application::Application>().unwrap().as_inner_mut().remove_tray(self.tray);
        true
    }
    fn on_close(&mut self, callback: Option<callbacks::Action>) {
        self.on_close = callback;
    }
}

impl TrayInner for CocoaTray {
    fn with_params(title: &str, menu: types::Menu) -> Box<Member<Self>> {
        use plygui_api::controls::HasLabel;

        let status_bar: cocoa_id = unsafe { NSStatusBar::systemStatusBar(ptr::null_mut()) };
        let mut t = Box::new(Member::with_inner(
            CocoaTray {
                tray: unsafe { status_bar.statusItemWithLength_(NSSquareStatusItemLength) },
                this: ptr::null_mut(),
                on_close: None,
            },
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        t.set_label(title);
        t.as_inner_mut().this = t.as_mut() as *mut Tray;

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

impl_all_defaults!(Tray);
