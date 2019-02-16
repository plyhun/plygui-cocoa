use super::common::*;

use core_graphics::base::{kCGBitmapByteOrderDefault, kCGImageAlphaLast};
use core_graphics::color_space::CGColorSpace;
use core_graphics::data_provider::CGDataProvider;
use core_graphics::image::CGImage;

use std::sync::Arc;

use external::image;

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        common::register_window_class("PlyguiImage", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const DEFAULT_PADDING: i32 = 6;
const BASE_CLASS: &str = "NSImageView";

pub type Image = Member<Control<CocoaImage>>;

#[repr(C)]
pub struct CocoaImage {
    base: common::CocoaControlBase<Image>,

    img: cocoa_id,
}

impl CocoaImage {
    fn install_image(&mut self, content: image::DynamicImage) {
        use image::GenericImageView;

        let size = content.dimensions();

        unsafe {
            let color_space = CGColorSpace::create_device_rgb();
            let provider = CGDataProvider::from_buffer(Arc::new(content.to_rgba().into_raw()));
            let cgimage = CGImage::new(size.0 as usize, size.1 as usize, 8, 32, 4 * size.0 as usize, &color_space, kCGBitmapByteOrderDefault | kCGImageAlphaLast, &provider, true, 0);

            self.img = msg_send![class!(NSImage), alloc];
            let size = NSSize::new(size.0 as f64, size.1 as f64);
            let () = msg_send![self.img, initWithCGImage:cgimage size:size];
            let () = msg_send![self.base.control, setImage:self.img];
        }
    }
    fn remove_image(&mut self) {
        unsafe {
            let () = msg_send![self.img, dealloc];
        }
    }
}

impl Drop for CocoaImage {
    fn drop(&mut self) {
        self.remove_image();
    }
}

impl ImageInner for CocoaImage {
    fn with_content(content: image::DynamicImage) -> Box<controls::Image> {
        let mut i = Box::new(Member::with_inner(
            Control::with_inner(
                CocoaImage {
                    base: common::CocoaControlBase::with_params(*WINDOW_CLASS),
                    img: nil,
                },
                (),
            ),
            MemberFunctions::new(_as_any, _as_any_mut, _as_member, _as_member_mut),
        ));
        let selfptr = i.as_mut() as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *i.as_inner_mut().as_inner_mut().base.control).set_ivar(common::IVAR, selfptr);
            let () = msg_send![i.as_inner_mut().as_inner_mut().base.control, setImageAlignment:0];
        }
        i.as_inner_mut().as_inner_mut().install_image(content);
        i
    }
    fn set_scale(&mut self, _member: &mut MemberBase, _control: &mut ControlBase, policy: types::ImageScalePolicy) {
        if self.scale() != policy {
            let scale = policy_to_nsscale(policy);
            unsafe {
                let () = msg_send![self.base.control, setImageScaling: scale];
            }
            self.base.invalidate();
        }
    }
    fn scale(&self) -> types::ImageScalePolicy {
        let scale = unsafe { msg_send![self.base.control, imageScaling] };
        nsscale_to_policy(scale)
    }
}

impl ControlInner for CocoaImage {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        self.base.invalidate();
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &controls::Container) {
        unsafe {
            self.base.on_removed_from_container();
        }
    }

    fn parent(&self) -> Option<&controls::Member> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&controls::Member> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut controls::Member> {
        self.base.root_mut()
    }

    #[cfg(feature = "markup")]
    fn fill_from_markup(&mut self, base: &mut MemberBase, control: &mut ControlBase, markup: &plygui_api::markup::Markup, registry: &mut plygui_api::markup::MarkupRegistry) {
        fill_from_markup_base!(self, base, markup, registry, Image, ["Image"]);
        //TODO image source
    }
}

impl HasNativeIdInner for CocoaImage {
    type Id = common::CocoaId;

    unsafe fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaImage {
    fn on_size_set(&mut self, base: &mut MemberBase, (width, height): (u16, u16)) -> bool {
        use plygui_api::controls::HasLayout;

        let this = base.as_any_mut().downcast_mut::<Image>().unwrap();
        this.set_layout_width(layout::Size::Exact(width));
        this.set_layout_width(layout::Size::Exact(height));
        self.base.invalidate();
        true
    }
}

impl HasVisibilityInner for CocoaImage {
    fn on_visibility_set(&mut self, _base: &mut MemberBase, value: types::Visibility) -> bool {
        self.base.on_set_visibility(value)
    }
}

impl MemberInner for CocoaImage {}

impl HasLayoutInner for CocoaImage {
    fn on_layout_changed(&mut self, _: &mut MemberBase) {
        self.base.invalidate();
    }
}

impl Drawable for CocoaImage {
    fn draw(&mut self, _member: &mut MemberBase, control: &mut ControlBase) {
        self.base.draw(control.coords, control.measured);
    }
    fn measure(&mut self, _member: &mut MemberBase, control: &mut ControlBase, parent_width: u16, parent_height: u16) -> (u16, u16, bool) {
        use std::cmp::max;

        let old_size = control.measured;
        control.measured = match control.visibility {
            types::Visibility::Gone => (0, 0),
            _ => unsafe {
                let mut label_size = (0, 0);
                let w = match control.layout.width {
                    layout::Size::MatchParent => parent_width as i32,
                    layout::Size::Exact(w) => w as i32,
                    layout::Size::WrapContent => {
                        let rep: cocoa_id = msg_send![self.img, representations];
                        let rep: cocoa_id = msg_send![rep, objectAtIndex:0];
                        let w: isize = msg_send![rep, pixelsWide];
                        let h: isize = msg_send![rep, pixelsHigh];
                        label_size = (w as u16, h as u16);
                        label_size.0 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                let h = match control.layout.height {
                    layout::Size::MatchParent => parent_height as i32,
                    layout::Size::Exact(h) => h as i32,
                    layout::Size::WrapContent => {
                        if label_size.1 < 1 {
                            let rep: cocoa_id = msg_send![self.img, representations];
                            let rep: cocoa_id = msg_send![rep, objectAtIndex:0];
                            let w: isize = msg_send![rep, pixelsWide];
                            let h: isize = msg_send![rep, pixelsHigh];
                            label_size = (w as u16, h as u16);
                        }
                        label_size.1 as i32 + DEFAULT_PADDING + DEFAULT_PADDING
                    }
                };
                (max(0, w) as u16, max(0, h) as u16)
            },
        };
        (control.measured.0, control.measured.1, control.measured != old_size)
    }
    fn invalidate(&mut self, _: &mut MemberBase, _: &mut ControlBase) {
        self.base.invalidate();
    }
}

fn policy_to_nsscale(i: types::ImageScalePolicy) -> u32 {
    // TODO NSImageScaling
    match i {
        types::ImageScalePolicy::CropCenter => 2,
        types::ImageScalePolicy::FitCenter => 3,
    }
}
fn nsscale_to_policy(i: u32) -> types::ImageScalePolicy {
    match i {
        3 => types::ImageScalePolicy::FitCenter,
        2 => types::ImageScalePolicy::CropCenter,
        _ => {
            println!("Unknown scale: {}", i);
            types::ImageScalePolicy::FitCenter
        }
    }
}
/*#[allow(dead_code)]
pub(crate) fn spawn() -> Box<controls::Control> {
    Image::with_label("").into_control()
}*/

extern "C" fn set_frame_size(this: &mut Object, _: Sel, param: NSSize) {
    unsafe {
        let sp = common::member_from_cocoa_id_mut::<Image>(this).unwrap();
        let () = msg_send![super(sp.as_inner_mut().as_inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        sp.call_on_size(param.width as u16, param.height as u16)
    }
}
impl_all_defaults!(Image);
