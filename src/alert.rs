use super::common::*;

const BASE_CLASS: &str = "NSAlert";

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe { register_window_class("PlyguiAlert", BASE_CLASS, |decl| {
        decl.add_method(sel!(anyButtonPressed:), button_pressed as extern "C" fn(&mut Object, Sel, cocoa_id));
    }) };
}

pub type Alert = Member<CocoaAlert>;

#[repr(C)]
pub struct CocoaAlert {
    control: cocoa_id,
    actions: Vec<(String, callbacks::Action, cocoa_id, Sel)>,
}

impl AlertInner for CocoaAlert {
    fn with_actions(content: types::TextContent, severity: types::AlertSeverity, mut actions: Vec<(String, callbacks::Action)>, parent: Option<&controls::Member>) -> Box<Member<Self>> {
        unsafe {
            let alert: cocoa_id = msg_send![WINDOW_CLASS.0, alloc];
            let alert: cocoa_id = msg_send![alert, init];
            
            let style = match severity {
                types::AlertSeverity::Info => 1,
                types::AlertSeverity::Warning => 0,
                types::AlertSeverity::Alert => 2,
            };
            let _ = msg_send![alert, setAlertStyle:style];
            
            let _ = match content {
                types::TextContent::Plain(text) => {
                    let text = NSString::alloc(cocoa::base::nil).init_str(&text);
                    msg_send![alert, setMessageText:text]
                },
                types::TextContent::LabelDescription(label, description) => {
                    let text = NSString::alloc(cocoa::base::nil).init_str(&description);
                    let _ = msg_send![alert, setInformativeText:text];
                    let text = NSString::alloc(cocoa::base::nil).init_str(&label);
                    msg_send![alert, setMessageText:text]
                }
            };
            
            let actions = actions.drain(..).enumerate().map(|(index, (name, action))| {
                let text = NSString::alloc(cocoa::base::nil).init_str(&name);
                let _ = msg_send![alert, addButtonWithTitle:text];
                let buttons: cocoa_id = msg_send![alert, buttons];
                let button: cocoa_id = msg_send![buttons, objectAtIndex:index];
                let old_target: cocoa_id = msg_send![button, target];
                let old_sel: Sel = msg_send![button, action];
                let _ = msg_send![button, setTarget:alert];
                let _ = msg_send![button, setAction:sel!(anyButtonPressed:)];
                (name, action, old_target, old_sel)
            }).collect::<Vec<_>>();
            
            let mut alert = Box::new(Member::with_inner(
                CocoaAlert { control: alert, actions: actions },
                MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
            ));
            
            let selfptr = alert.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
            (&mut *alert.as_inner_mut().control).set_ivar(common::IVAR, selfptr);
            println!("in {:?} = {}", selfptr, alert.as_inner_mut().actions.len());
            
            match parent {
                Some(parent) => {
                    let window: cocoa_id = common::parent_cocoa_id(parent.native_id() as cocoa_id, true).unwrap();
                    let _ = msg_send![alert.as_inner_mut().control, beginSheetModalForWindow:window completionHandler:nil];
                },
                None => {
                    let _ = msg_send![alert.as_inner_mut().control, runModal];
                }
            }
            alert
        }
    }
    fn severity(&self) -> types::AlertSeverity {
        let style = unsafe { msg_send![self.control, alertStyle] };
        match style {
            0 => types::AlertSeverity::Warning,
            1 => types::AlertSeverity::Info,
            2 => types::AlertSeverity::Alert,
            _ => unreachable!(),
        }
    }
}

impl HasLabelInner for CocoaAlert {
    fn label(&self) -> ::std::borrow::Cow<'_, str> {
        unsafe {
            let title: cocoa_id = msg_send![self.control, messageText];
            let title = msg_send![title, UTF8String];
            Cow::Owned(ffi::CString::from_raw(title).into_string().unwrap())
        }
    }
    fn set_label(&mut self, _: &mut MemberBase, label: &str) {
        unsafe {
            let label = NSString::alloc(cocoa::base::nil).init_str(label);
            let () = msg_send![self.control, setMesssageText: label];
            let () = msg_send![label, release];
        }
    }
}

impl MemberInner for CocoaAlert {
    type Id = common::CocoaId;

    fn size(&self) -> (u16, u16) {
        let frame: NSRect = unsafe { msg_send![self.control, frame] };
        (frame.size.width as u16, frame.size.height as u16)
    }

    fn on_set_visibility(&mut self, base: &mut MemberBase) {
        unsafe {
            let () = if types::Visibility::Visible == base.visibility {
                msg_send![self.control, setIsVisible: YES]
            } else {
                msg_send![self.control, setIsVisible: NO]
            };
        }
    }

    unsafe fn native_id(&self) -> Self::Id {
        self.control.into()
    }
}

extern "C" fn button_pressed(this: &mut Object, _: Sel, param: cocoa_id) {
    unsafe {
        let alert = common::member_from_cocoa_id_mut::<Alert>(this).unwrap();
        let title = {
            let title: cocoa_id = msg_send![param, title];
            let title = msg_send![title, UTF8String];
            ffi::CString::from_raw(title).into_string().unwrap()
        }; 
        alert.as_inner_mut().actions.iter_mut().filter(|a| a.0 == title).for_each(|a| {
            let alert2 = common::member_from_cocoa_id_mut::<Alert>(this).unwrap();
            (a.1.as_mut())(alert2);
            let _ = msg_send![a.2, performSelector:a.3 withObject:param];
        });
        
        println!("out {:?} = {}", common::cast_cocoa_id_to_ptr(this).unwrap(), alert.as_inner_mut().actions.len());
        
        mem::forget(title);
    }
}

impl_all_defaults!(Alert);
