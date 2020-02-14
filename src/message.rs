use crate::common::{self, *};

const BASE_CLASS: &str = "NSAlert";

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        register_window_class("PlyguiMessage", BASE_CLASS, |decl| {
            decl.add_method(sel!(anyButtonPressed:), button_pressed as extern "C" fn(&mut Object, Sel, cocoa_id));
        })
    };
}

pub type Message = AMember<AMessage<CocoaMessage>>;

#[repr(C)]
pub struct CocoaMessage {
    control: cocoa_id,
    parent: cocoa_id,
    actions: Vec<(String, callbacks::Action, cocoa_id, Sel)>,
}

impl MessageInner for CocoaMessage {
    fn with_actions(content: types::TextContent, severity: types::MessageSeverity, actions: Vec<(String, callbacks::Action)>, parent: Option<&dyn controls::Member>) -> Box<dyn controls::Message> {
        unsafe {
            let alert: cocoa_id = msg_send![WINDOW_CLASS.0, alloc];
            let alert: cocoa_id = msg_send![alert, init];

            let style = match severity {
                types::MessageSeverity::Info => 1,
                types::MessageSeverity::Warning => 0,
                types::MessageSeverity::Alert => 2,
            };
            let _ = msg_send![alert, setAlertStyle: style];

            let _ = match content {
                types::TextContent::Plain(text) => {
                    let text = NSString::alloc(cocoa::base::nil).init_str(&text);
                    msg_send![alert, setMessageText: text]
                }
                types::TextContent::LabelDescription(label, description) => {
                    let text = NSString::alloc(cocoa::base::nil).init_str(&description);
                    let _ = msg_send![alert, setInformativeText: text];
                    let text = NSString::alloc(cocoa::base::nil).init_str(&label);
                    msg_send![alert, setMessageText: text]
                }
            };

            let actions = actions
                .into_iter()
                .enumerate()
                .map(|(index, (name, action))| {
                    let text = NSString::alloc(cocoa::base::nil).init_str(&name);
                    let _ = msg_send![alert, addButtonWithTitle: text];
                    let buttons: cocoa_id = msg_send![alert, buttons];
                    let button: cocoa_id = msg_send![buttons, objectAtIndex: index];
                    let old_target: cocoa_id = msg_send![button, target];
                    let old_sel: Sel = msg_send![button, action];
                    let _ = msg_send![button, setTarget: alert];
                    let _ = msg_send![button, setAction: sel!(anyButtonPressed:)];
                    (name, action, old_target, old_sel)
                })
                .collect::<Vec<_>>();

            let parent = match parent {
                Some(parent) => common::parent_cocoa_id(parent.native_id() as cocoa_id, true).unwrap(),
                None => 0 as cocoa_id,
            };

            let mut alert = Box::new(AMember::with_inner(
                AMessage::with_inner(
                    CocoaMessage {
                        control: alert,
                        actions: actions,
                        parent: parent,
                    }
                )
            ));

            let selfptr = alert.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
            (&mut *alert.inner_mut().inner_mut().control).set_ivar(common::IVAR, selfptr);

            alert
        }
    }
    fn start(self) -> Result<String, ()> {
        let mut pressed: NSInteger = match self.parent as usize {
            0 => unsafe { msg_send![self.control, runModal] },
            _ => unsafe {
                let completion_handler = ConcreteBlock::new(move |return_code: NSInteger| {
                    let app: cocoa_id = msg_send![class!(NSApplication), sharedApplication];
                    msg_send![app, stopModalWithCode: return_code];
                });
                let completion_handler = completion_handler.copy();
                let completion_handler: &Block<(NSInteger,), ()> = &completion_handler;

                let _ = msg_send![self.control, beginSheetModalForWindow:self.parent completionHandler:completion_handler];
                let app: cocoa_id = msg_send![class!(NSApplication), sharedApplication];
                let window: cocoa_id = msg_send![self.control, window];
                msg_send![app, runModalForWindow: window]
            },
        };
        pressed -= 1000;
        self.actions.get(pressed as usize).map(|a| a.0.clone()).ok_or(())
    }
    fn severity(&self) -> types::MessageSeverity {
        let style = unsafe { msg_send![self.control, alertStyle] };
        match style {
            0 => types::MessageSeverity::Warning,
            1 => types::MessageSeverity::Info,
            2 => types::MessageSeverity::Alert,
            _ => unreachable!(),
        }
    }
}

impl HasLabelInner for CocoaMessage {
    fn label(&self, _: &MemberBase) -> Cow<str> {
        unsafe {
            let title: cocoa_id = msg_send![self.control, messageText];
            let title = msg_send![title, UTF8String];
            Cow::Owned(ffi::CString::from_raw(title).into_string().unwrap())
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: Cow<str>) {
        unsafe {
            let label = NSString::alloc(cocoa::base::nil).init_str(&label);
            let () = msg_send![self.control, setMesssageText: label];
            let () = msg_send![label, release];
        }
    }
}

impl HasNativeIdInner for CocoaMessage {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.control.into()
    }
}

impl MemberInner for CocoaMessage {}

extern "C" fn button_pressed(this: &mut Object, _: Sel, param: cocoa_id) {
    unsafe {
        let alert = common::member_from_cocoa_id_mut::<Message>(this).unwrap();
        let title = {
            let title: cocoa_id = msg_send![param, title];
            let title = msg_send![title, UTF8String];
            ffi::CString::from_raw(title).into_string().unwrap()
        };
        alert.inner_mut().inner_mut().actions.iter_mut().filter(|a| a.0 == title).for_each(|a| {
            let alert2 = common::member_from_cocoa_id_mut::<Message>(this).unwrap();
            (a.1.as_mut())(alert2);
            let _ = msg_send![a.2, performSelector:a.3 withObject:param];
        });

        mem::forget(title);
    }
}
