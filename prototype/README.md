# PROTOTYPE: main window + video through libmpv's render API (throwaway)

Answers GitHub issue #7 ("Prototype: main window with video through the render API"). Not production code. Do not merge.

Run: `./run.sh [shot.png]` (needs Nix; headless via Xvfb + Mesa llvmpipe). Env: `PROTO_RUST_SEEKBAR=1` swaps the C++ SeekBar for the Rust one; `PROTO_VIDEO=<file|url>` plays a real file (default is mpv's lavfi `testsrc`). `PROTO_HEADLESS=` (empty) runs on the real display.

## What it builds (Qt 6.11.2, mpv 0.41 / libmpv API 2.5, cxx-qt 0.10.0, libmpv2 6.0.0, all from nixpkgs/crates.io)

1. **Original layout.** `src/ui/mainwindow.ui` goes through `uic` unchanged. Its QSS, SVG icons (`src/rsclist.qrc`) and 7 promoted custom widgets (`src/widgets/{customslider,customlabel,customlineedit,customsplitter,openbutton,indexbutton,seekbar}.cpp`) compile unchanged on Qt 6. `PlaylistWidget` is a stub (the real one depends on `BakaEngine`/`MpvHandler`).
2. **Video.** `cpp/mpvglwidget.cpp` is a `QOpenGLWidget` placed in `mpvFrame` (replaces `wid` embedding). `src/video.rs` owns mpv and the render context (libmpv2, OpenGL backend) and renders into the widget's FBO.
3. **Rust SeekBar.** `src/seekbar.rs` is `SeekBar` as a cxx-qt `#[base = QSlider]` object: it overrides `paintEvent`, calls the base `paintEvent`, then draws ticks with `QPainter`.

## Results (headless, Linux x86_64 only)

- Window renders: `shots/stage1-offscreen.png`. With video playing: `shots/stage2-video.png`.
- Rust SeekBar vs C++ SeekBar, same state (enabled, 1/3 position, ticks at 25/50/75%): full-window screenshots are pixel-identical (`shots/seek-cpp.png` vs `shots/seek-rust.png`, PIL `ImageChops.difference(...).getbbox()` is `None`). Crop: `shots/seek-rust-crop.png`.
- `paintEvent` override with a base-class call works from Rust. Mouse-event overrides, signals/slots to other widgets and `.ui`-promoted Rust widgets were not tried.
- Size: about 340 lines of Rust/C++/build glue for the spike, of which 85 lines are the SeekBar and 71 are the video path.

## Findings that matter for the decision

- Every event or painter type must be declared by hand in the bridge (`QPaintEvent`, `QWidget`, `QSlider`). `QPainter` on a widget needed a 13-line C++ shim (`cpp/rustseekbar_shim.cpp`). Upcasting `RustSeekBar` to `QSlider*` needed `unsafe`.
- `uic` is not wrapped by cxx-qt-build; `build.rs` runs it directly (`PROTO_UIC`).
- `CxxQtBuilder::qt_module` must list `Gui` explicitly (the original widgets use `QMouseEvent`, `QPaintEvent`). `qt_module("Svg")` panics under nixpkgs ("Could not find a prl path"): nixpkgs splits Qt modules into separate prefixes and cxx-qt-build only reads the `qmake` prefix. SVG icons still work because only the Svg plugin is needed at runtime (`QT_PLUGIN_PATH`).
- The binary links but does not run until the final crate references `cxx-qt-lib` (otherwise `cxx_qt_init_crate_cxx_qt_lib` is undefined).
- The render API needs the update callback to hop to the GUI thread (`QMetaObject::invokeMethod(..., Qt::QueuedConnection)`), and the render context must be freed with the GL context current (done in the widget destructor).

## macOS (Apple Silicon, nixpkgs Qt 6.11.2) findings from the owner's runs

- nixpkgs builds Qt on macOS as frameworks; cxx-qt-build passes only `-I`, so the shell must add `-F <qt>/lib` (flake does).
- The root `.gitignore` rule `build*` hid `build.rs` until `09a7db5`; check new files with `git check-ignore -v`.
- libmpv needs `LC_NUMERIC=C`. Qt adopts the system locale, so `mpv_create` returned NULL (`mpv init: Null`) on a non-C locale. `proto_run` now calls `setlocale(LC_NUMERIC, "C")`, as the original `src/main.cpp` does. Reproduced on Linux with `LC_ALL=C.utf8`.
- Qt 6 deprecation warnings in the original widgets (all still compile): `QMouseEvent::x()`, `globalPos()`, `globalX()` in `customslider.cpp`, `indexbutton.cpp`, `seekbar.cpp`. Port to `position()` / `globalPosition()`.
- Fidelity note: the QSS names `Noto Sans`/`FreeSans`; Qt warns the family is missing on this Mac. Fonts must be bundled or fidelity screenshots will differ per OS.

## Not verified

- Windows and macOS: not built or run. Expected blockers from the research: ANGLE or WGL interop for hardware decode on Windows, CGL (deprecated) on macOS, `QtWinExtras` replacement.
- Native Wayland and X11 with a real GPU: only Xvfb with llvmpipe was used.
- Hardware decoding (`hwdec=no`), HDR, HiDPI (`devicePixelRatioF` is passed but untested), seek and scrub latency, real files, audio, `OverlayHandler` text overlays.
- Fonts: the screenshots use whatever fonts the nix Qt finds, so they are not a fidelity baseline. A C++ Qt 5 build of the original for comparison was not made.
- Qt 5 vs Qt 6 visual differences: no Qt 5 baseline exists yet.
