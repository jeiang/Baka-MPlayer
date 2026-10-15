# Research: Rust bindings that can drive Qt Widgets
Type: research
Status: open
Blocked by:

## Question

Which Rust-to-Qt routes can build a Qt Widgets UI (not QML) with custom widgets, `.ui` files, QSS, SVG icons, and subclassed events, as used in `src/ui/` and `src/widgets/`? Cover cxx-qt, qmetaobject-rs, rust-qt/ritual, and a thin C++ shim crate. For each: Qt 5/6 support, Widgets coverage, maintenance state, Windows/macOS/Linux build story, and whether subclassing (paintEvent, mouse events) is possible. Primary sources only (repos, docs, release notes).
