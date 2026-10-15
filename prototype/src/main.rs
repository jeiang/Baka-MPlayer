// PROTOTYPE (throwaway). See README.md.
#[cxx_qt::bridge]
mod ffi {}

extern "C" {
    fn proto_run(argc: i32, argv: *mut *mut std::os::raw::c_char) -> i32;
}

fn main() {
    // Reference cxx-qt-lib so its static initializer library is linked.
    let _ = cxx_qt_lib::QString::from("");
    let args: Vec<std::ffi::CString> = std::env::args().map(|a| std::ffi::CString::new(a).unwrap()).collect();
    let mut ptrs: Vec<*mut std::os::raw::c_char> = args.iter().map(|a| a.as_ptr() as *mut _).collect();
    ptrs.push(std::ptr::null_mut());
    std::process::exit(unsafe { proto_run(args.len() as i32, ptrs.as_mut_ptr()) });
}
