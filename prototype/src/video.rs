// PROTOTYPE: libmpv render API driven from Rust (libmpv2 crate, OpenGL backend).
use libmpv2::render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType};
use libmpv2::Mpv;
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::Mutex;

extern "C" {
    fn proto_request_update(widget: *mut c_void);
    fn proto_get_proc_address(widget: *mut c_void, name: *const c_char) -> *mut c_void;
}

struct Gl(*mut c_void); // the MpvGLWidget; only touched on the GUI thread via C++
struct Send_(usize);

struct Video {
    // Mpv is leaked so the RenderContext can borrow it for 'static; dropped explicitly on shutdown.
    mpv: &'static Mpv,
    render: RenderContext<'static>,
}
// Only ever used from the GUI thread; the Mutex just satisfies `static`.
unsafe impl Send for Video {}
static VIDEO: Mutex<Option<Video>> = Mutex::new(None);

fn get_proc(ctx: &Gl, name: &str) -> *mut c_void {
    let c = CString::new(name).unwrap();
    unsafe { proto_get_proc_address(ctx.0, c.as_ptr()) }
}

#[no_mangle]
pub extern "C" fn proto_mpv_init(widget: *mut c_void) {
    let mpv: &'static Mpv = Box::leak(Box::new(
        Mpv::with_initializer(|i| {
            i.set_property("vo", "libmpv")?;
            i.set_property("hwdec", "no")?;
            i.set_property("ao", "null")?;
            i.set_property("keep-open", "yes")?;
            Ok(())
        })
        .expect("mpv init"),
    ));
    let mut render = mpv
        .create_render_context([
            RenderParam::ApiType(RenderParamApiType::OpenGl),
            RenderParam::InitParams(OpenGLInitParams { get_proc_address: get_proc, ctx: Gl(widget) }),
        ])
        .expect("render context");
    let w = Send_(widget as usize);
    render.set_update_callback(move || unsafe { proto_request_update(w.0 as *mut c_void) });
    let src = std::env::var("PROTO_VIDEO").unwrap_or_else(|_| "av://lavfi:testsrc=size=640x360:rate=30".into());
    mpv.command("loadfile", &[&src]).expect("loadfile");
    *VIDEO.lock().unwrap() = Some(Video { mpv, render });
}

#[no_mangle]
pub extern "C" fn proto_mpv_paint(fbo: i32, w: i32, h: i32) {
    if let Some(v) = VIDEO.lock().unwrap().as_ref() {
        let _ = v.render.update();
        v.render.render::<Gl>(fbo, w, h, true).ok();
    }
}

#[no_mangle]
pub extern "C" fn proto_mpv_shutdown() {
    if let Some(v) = VIDEO.lock().unwrap().take() {
        drop(v.render); // before mpv_terminate_destroy, per render.h
        let _ = v.mpv; // leaked on purpose in the prototype
    }
}

#[allow(dead_code)]
fn _unused(_: &CStr) {}
