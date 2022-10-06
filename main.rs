use gtk::glib;
use gtk::prelude::*;

glib::wrapper! {
    pub struct FemtovgCanvas(ObjectSubclass<imp::FemtovgCanvas>)
        @extends gtk::Widget, gtk::GLArea;
}

impl Default for FemtovgCanvas {
    fn default() -> Self {
        glib::Object::new(&[]).unwrap()
    }
}

mod imp {
    use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct FemtovgCanvas {
        canvas: RefCell<Option<femtovg::Canvas<femtovg::renderer::OpenGl>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FemtovgCanvas {
        const NAME: &'static str = "FemtovgCanvas";
        type Type = super::FemtovgCanvas;
        type ParentType = gtk::GLArea;
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            let obj = unsafe { obj.as_ref() };
            obj.set_has_stencil_buffer(true);
        }
    }

    impl ObjectImpl for FemtovgCanvas {}

    impl WidgetImpl for FemtovgCanvas {
        fn unrealize(&self, widget: &Self::Type) {
            widget.make_current();
            self.canvas.replace(None);
            self.parent_unrealize(widget);
        }
    }

    impl GLAreaImpl for FemtovgCanvas {
        fn resize(&self, widget: &Self::Type, width: i32, height: i32) {
            self.ensure_canvas();
            if let Some(canvas) = self.canvas.borrow_mut().as_mut() {
                canvas.set_size(width as u32, height as u32, widget.scale_factor() as f32);
            }
        }
        fn render(&self, widget: &Self::Type, _context: &gtk::gdk::GLContext) -> bool {
            use femtovg::{Color, Paint, Path};

            fn to_fvg(c: &gdk::RGBA) -> Color {
                Color::rgbaf(c.red(), c.green(), c.blue(), c.alpha())
            }

            let ctx = widget.style_context();
            let fg = to_fvg(
                &ctx.lookup_color("theme_fg_color")
                    .expect("no theme_fg_color"),
            );
            if let Some(canvas) = self.canvas.borrow_mut().as_mut() {
                let width = widget.width();
                let height = widget.height();
                canvas.clear_rect(0, 0, width as u32, height as u32, Color::rgba(0, 0, 0, 0));
                let mut path = Path::new();
                path.move_to(20., 20.);
                path.line_to(20., 80.);
                path.move_to(20., 50.);
                path.line_to(40., 50.);
                path.move_to(40., 20.);
                path.line_to(40., 80.);
                path.move_to(50., 20.);
                path.line_to(70., 20.);
                path.move_to(50., 80.);
                path.line_to(70., 80.);
                path.move_to(60., 20.);
                path.line_to(60., 80.);
                let mut paint = Paint::color(fg);
                paint.set_line_width(2.);
                canvas.stroke_path(&mut path, paint);
                canvas.flush();
            }
            true
        }
    }
    impl FemtovgCanvas {
        fn ensure_canvas(&self) {
            use femtovg::{renderer, Canvas};
            use glow::HasContext;

            if self.canvas.borrow().is_some() {
                return;
            }
            let widget = self.instance();
            widget.attach_buffers();

            static LOAD_FN: fn(&str) -> *const std::ffi::c_void =
                |s| epoxy::get_proc_addr(s) as *const _;
            let mut renderer = unsafe {
                renderer::OpenGl::new_from_function(LOAD_FN).expect("Cannot create renderer")
            };

            let fbo_id = unsafe {
                let ctx = glow::Context::from_loader_function(LOAD_FN);
                let id = ctx.get_parameter_i32(glow::DRAW_FRAMEBUFFER_BINDING) as u32;
                assert!(id != 0);
                ctx.bind_framebuffer(glow::FRAMEBUFFER, None);
                // TODO - transmute can be removed when a new version of glow is released with this
                // patch: https://github.com/grovesNL/glow/pull/211
                std::mem::transmute(id)
            };
            renderer.set_screen_target(Some(fbo_id));
            let canvas = Canvas::new(renderer).expect("Cannot create canvas");
            self.canvas.replace(Some(canvas));
        }
    }
}

fn main() {
    init_epoxy();

    let application =
        gtk::Application::new(Some("io.github.jf2048.FemtoVGCanvas"), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some("FemtoVG Canvas"));
    window.set_default_size(400, 400);

    window.set_child(Some(&FemtovgCanvas::default()));

    window.show();
}

pub fn init_epoxy() {
    #[cfg(target_os = "macos")]
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.0.dylib") }.unwrap();
    #[cfg(all(unix, not(target_os = "macos")))]
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
    #[cfg(windows)]
    let library = libloading::os::windows::Library::open_already_loaded("libepoxy-0.dll").unwrap();

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(std::ptr::null())
    });
}
