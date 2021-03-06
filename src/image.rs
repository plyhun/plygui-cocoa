use crate::common::{self, *};

lazy_static! {
    static ref WINDOW_CLASS: common::RefClass = unsafe {
        common::register_window_class("PlyguiImage", BASE_CLASS, |decl| {
            decl.add_method(sel!(setFrameSize:), set_frame_size as extern "C" fn(&mut Object, Sel, NSSize));
        })
    };
}

const DEFAULT_PADDING: i32 = 6;
const BASE_CLASS: &str = "NSImageView";

pub type Image = AMember<AControl<AImage<CocoaImage>>>;

#[repr(C)]
pub struct CocoaImage {
    base: common::CocoaControlBase<Image>,

    img: cocoa_id,
}

impl CocoaImage {
    fn install_image(&mut self, content: image::DynamicImage) {
        unsafe {
            self.img = common::image_to_native(&content);
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

impl<O: controls::Image> NewImageInner<O> for CocoaImage {
    fn with_uninit_params(ptr: &mut mem::MaybeUninit<O>, content: image::DynamicImage) -> Self {
        let mut i = CocoaImage {
            base: common::CocoaControlBase::with_params(*WINDOW_CLASS, set_frame_size_inner::<O>),
            img: nil,
        };
        let selfptr = ptr as *mut _ as *mut ::std::os::raw::c_void;
        unsafe {
            (&mut *i.base.control).set_ivar(common::IVAR, selfptr);
            let () = msg_send![i.base.control, setImageAlignment:0];
        }
        i.install_image(content);
        i
    }
}
impl ImageInner for CocoaImage {
    fn with_content(content: image::DynamicImage) -> Box<dyn controls::Image> {
        let mut b: Box<mem::MaybeUninit<Image>> = Box::new_uninit();
        let ab = AMember::with_inner(
            AControl::with_inner(
                AImage::with_inner(
                    <Self as NewImageInner<Image>>::with_uninit_params(b.as_mut(), content)
                )
            ),
        );
        unsafe {
	        b.as_mut_ptr().write(ab);
	        b.assume_init()
        }
    }
    fn set_scale(&mut self, _member: &mut MemberBase, policy: types::ImageScalePolicy) {
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
impl HasImageInner for CocoaImage {
    fn image(&self, _: &MemberBase) -> Cow<image::DynamicImage> {
        todo!() //Cow::Owned(unsafe { common::native_to_image(self.img) })
    }
    fn set_image(&mut self, _: &mut MemberBase, arg0: Cow<image::DynamicImage>) {
        self.install_image(arg0.into_owned())
    }
}
impl ControlInner for CocoaImage {
    fn on_added_to_container(&mut self, member: &mut MemberBase, control: &mut ControlBase, _parent: &dyn controls::Container, _x: i32, _y: i32, pw: u16, ph: u16) {
        self.measure(member, control, pw, ph);
        self.base.invalidate();
    }
    fn on_removed_from_container(&mut self, _: &mut MemberBase, _: &mut ControlBase, _: &dyn controls::Container) {
        unsafe {
            self.base.on_removed_from_container();
        }
    }

    fn parent(&self) -> Option<&dyn controls::Member> {
        self.base.parent()
    }
    fn parent_mut(&mut self) -> Option<&mut dyn controls::Member> {
        self.base.parent_mut()
    }
    fn root(&self) -> Option<&dyn controls::Member> {
        self.base.root()
    }
    fn root_mut(&mut self) -> Option<&mut dyn controls::Member> {
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

    fn native_id(&self) -> Self::Id {
        self.base.control.into()
    }
}

impl HasSizeInner for CocoaImage {
    fn on_size_set(&mut self, _: &mut MemberBase, _: (u16, u16)) -> bool {
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
impl Spawnable for CocoaImage {
    fn spawn() -> Box<dyn controls::Control> {
        Self::with_content(image::DynamicImage::new_luma8(0, 0)).into_control()
    }
}
extern "C" fn set_frame_size(this: &mut Object, sel: Sel, param: NSSize) {
    unsafe {
        let b = common::member_from_cocoa_id_mut::<Image>(this).unwrap();
        let b2 = common::member_from_cocoa_id_mut::<Image>(this).unwrap();
        (b.inner().inner().inner().base.resize_handler)(b2, sel, param)
    }
}
extern "C" fn set_frame_size_inner<O: controls::Image>(this: &mut Image, _: Sel, param: NSSize) {
    unsafe {
        let () = msg_send![super(this.inner_mut().inner_mut().inner_mut().base.control, Class::get(BASE_CLASS).unwrap()), setFrameSize: param];
        this.call_on_size::<O>(param.width as u16, param.height as u16)
    }
}
