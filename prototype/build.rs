// PROTOTYPE (throwaway): builds the original mainwindow.ui (via uic) against the original
// custom widgets, plus a Rust-owned libmpv render path. See README.md.
use cxx_qt_build::CxxQtBuilder;
use std::{env, path::PathBuf, process::Command};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uic = env::var("PROTO_UIC").expect("run inside `nix develop path:.` (sets PROTO_UIC)");
    let ui = Command::new(uic)
        .args(["../src/ui/mainwindow.ui", "-o"])
        .arg(out.join("ui_mainwindow.h"))
        .status()
        .unwrap();
    assert!(ui.success());

    let widgets = ["customslider", "customlabel", "customlineedit", "customsplitter", "openbutton", "indexbutton", "seekbar"];
    let mut cpp: Vec<String> = vec!["cpp/app.cpp".into(), "cpp/stubs/widgets/playlistwidget.h".into(), "cpp/mpvglwidget.h".into(), "cpp/rustseekbar_shim.cpp".into(), "cpp/mpvglwidget.cpp".into()];
    for w in widgets {
        cpp.push(format!("../src/widgets/{w}.h"));
        cpp.push(format!("../src/widgets/{w}.cpp"));
    }

    let mut b = CxxQtBuilder::new()
        .qt_module("Widgets")
        .qt_module("Gui")
        .qt_module("OpenGLWidgets")
        .qrc("../src/rsclist.qrc")
        .files(["src/main.rs", "src/seekbar.rs"])
        .cpp_files(cpp);
    unsafe {
        b = b.cc_builder(move |cc| {
            cc.include(&out).include("cpp/stubs").include("../src").include("cpp").flag_if_supported("-std=c++17");
        });
    }
    b.build();
}
