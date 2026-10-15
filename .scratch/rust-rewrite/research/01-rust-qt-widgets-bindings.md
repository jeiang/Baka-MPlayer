# Research 01: Rust bindings that can drive Qt Widgets

Ticket: `../issues/01-research-rust-qt-widgets-bindings.md`
Date: 2026-10-15
Method: shallow clones of the project repos plus `crates.io` metadata. Every claim cites a path in a cloned repo at the stated commit. Tags: **[V]** verified in a primary source, **[I]** inference from primary sources, **[U]** not verified.

Source commits read:

| Repo | Commit | Commit date |
|---|---|---|
| github.com/KDAB/cxx-qt | `f62f057` (main, after v0.10.0) | 2026-09-29 |
| github.com/woboq/qmetaobject-rs | `1f4d09a` | 2026-04-13 |
| github.com/rust-qt/ritual | `51b38ca` | 2021-05-18 |
| github.com/qt/qttools (UiTools) | HEAD on 2026-10-15 | n/a |
| github.com/qt/qtbase (uic) | HEAD on 2026-10-15 | n/a |

## 1. What the C++ UI needs (grounding)

Read from `src/ui/`, `src/widgets/`, `src/*.pro`. All [V].

- 9 `.ui` files; 8 of them are `QDialog`, 1 is `QMainWindow`. Standard widgets used: QPushButton, QLabel, QMenu (17), QLineEdit, QCheckBox, QGroupBox, QComboBox, QSpinBox, QPlainTextEdit, QRadioButton, QTabWidget, QTableWidget, QSplitter, QProgressBar, QMenuBar, QKeySequenceEdit, QFrame, layouts.
- 8 promoted custom widgets in `mainwindow.ui` (`<customwidgets>`): CustomSlider (QSlider), OpenButton and IndexButton (QPushButton), SeekBar (QSlider), CustomLabel (QLabel), CustomSplitter (QSplitter), PlaylistWidget (QListWidget), CustomLineEdit (QLineEdit). Each carries a `<header>widgets/x.h</header>`.
- Event overrides declared in headers: `paintEvent` (IndexButton, SeekBar, both use `QPainter`), `mousePressEvent` (CustomLabel, CustomSlider, DimDialog, OpenButton), `mouseMoveEvent` (IndexButton, SeekBar), `keyPressEvent` (CustomLineEdit), `contextMenuEvent` and `dropEvent` (PlaylistWidget). `MainWindow` overrides `dragEnterEvent`, `dropEvent`, `keyPressEvent`, `leaveEvent`, `mouse{DoubleClick,Move,Press,Release}Event`, `resizeEvent`, `wheelEvent`, and `eventFilter` (installed on `mpvFrame`).
- 25 `Q_OBJECT` classes, about 19 `signals:`/`slots:` blocks.
- QSS: 5 `setStyleSheet` call sites (`mainwindow.cpp`, `overlayhandler.cpp`, `dimdialog.cpp`). One uses `background-image:url(:/img/...)`, so QSS needs the Qt resource system (`.qrc`).
- SVG icons in `src/img` (QtSvg is in `QT += core gui network svg`).
- `winId()` of `mpvFrame` is handed to mpv (`bakaengine.cpp:19`), and `Util::SetAlwaysOnTop(winId(), ...)` is used. So the binding must expose native window handles.
- `QTranslator` plus `.ts/.qm` and `.qrc` for 13 translations.
- Total in `ui/` and `widgets/`: about 2.7k LOC of `.cpp`.

Capability checklist derived from this: (a) Widgets classes, (b) subclassing with virtual overrides (paint, mouse, key, drop, context menu, resize, wheel), (c) QObject signals and slots with new signals on subclasses, (d) `.ui` with promoted widgets, (e) QSS and `.qrc`, (f) QtSvg, (g) `winId()`, (h) QTranslator, (i) Qt 5 and/or 6, (j) Windows, macOS, Linux builds.

## 2. cxx-qt (KDAB)

**Maintenance [V].** `crates.io`: cxx-qt 0.10.0, published 2026-08-24; about 126k downloads. `CHANGELOG.md` lists releases 0.8.1 (2026-02-16), 0.9.0 (2026-06-23), 0.9.1 (2026-07-03), 0.10.0 (2026-08-24), plus unreleased fixes. Commits continue after 0.10.0 (2026-09-29). `README.md`: "CXX-Qt is in early development and the API changes frequently."

**Qt versions [V].** `README.md` "Supported Qt Versions": Qt 5.15 LTS and all Qt 6; "Versions below Qt 5.15 LTS are explicitly unsupported". CMake defaults to Qt 6; `USE_QT5=ON` selects Qt 5. For Cargo, the env var `QT_VERSION_MAJOR` chooses (`crates/cxx-qt-build/src/lib.rs`, doc near line 340).

**Widgets coverage [V].**
- `cxx-qt-lib` features are `qt_gui`, `qt_qml`, `qt_quickcontrols` (`crates/cxx-qt-lib/Cargo.toml`). No Widgets feature. `src/gui/` has QPainter, QPainterPath, QPen, QColor, QFont, QImage, QRegion, QPolygon, and others, but no widget classes.
- The only Widgets binding in the repo is `QApplication` in `crates/cxx-qt-lib-extras` (`src/gui/qapplication.rs`; its `build.rs` calls `.qt_module("Widgets")`). It also has QCommandLineParser, QElapsedTimer, QEventLoop.
- So QPushButton, QSlider, QMenu, QSplitter, QListWidget, QMainWindow and the rest have **no ready bindings**. They must be declared by hand with CXX `extern "C++"` blocks, one method at a time.
- `CxxQtBuilder::qt_module("Widgets")` links extra Qt modules (`crates/cxx-qt-build/src/lib.rs` ~line 594, doc says: `Specify their names without the Qt prefix, for example "Widgets"`).
- The parser test input `crates/cxx-qt-gen/test_inputs/inheritance.rs` contains `extern "C++Qt" { include!(<QtWidgets/QPushButton>); #[qobject] type QPushButton; }` and `#[base = QPushButton] type QPushButtonChild;`. This shows the code generator accepts a Widgets base class. It is a generator input fixture, not an end-to-end widget example [V for the fixture, I for "works end to end"].
- No widget example in `examples/` (all are QML: `qml_basics`, `qml_features`, `qml_minimal`, `todo_app`, `demo_threading`, `span-inspector`, ...) [V].

**Subclassing [V + I].**
- Book `book/src/bridge/extern_rustqt.md`, "base attribute": "The base class must inherit from `QObject` (directly or indirectly)". QWidget qualifies.
- `book/src/concepts/inheritance.md`: `#[inherit]` calls base methods; `#[cxx_override]`, `#[cxx_virtual]`, `#[cxx_final]` generate the C++ `override`/`virtual`/`final` modifiers. The worked example overrides `QAbstractListModel::data`.
- [I] The same mechanism should let a Rust-backed QObject based on `QPushButton`/`QSlider` override `paintEvent(QPaintEvent*)` and `mousePressEvent(QMouseEvent*)`: the generated C++ class is a real subclass, so protected virtuals are overridable. The event types (QPaintEvent, QMouseEvent, QKeyEvent, QDropEvent, ...) are not bound in `cxx-qt-lib`, so each must be declared by hand, and `QPainter(this)` needs `QPainter` constructed on a `QWidget*` (cxx-qt-lib has `QPainter`, but the paint-device constructor path is [U]). No upstream example shows this; it would need a prototype.
- Constructors with base-class arguments (a `QWidget* parent`) go through the `Constructor` trait (`book/src/concepts/inheritance.md`, `book/src/bridge/traits.md`).
- Escape hatch [V]: `CxxQtBuilder::cpp_file(...)` ("allows building QObject C++ subclasses besides the ones autogenerated by cxx-qt", `lib.rs` ~line 347, with moc run on them). So hand-written C++ widgets can live in the same crate. This is the shim route inside cxx-qt.

**Signals and slots [V].** `#[qsignal]`, `#[qinvokable]`, `#[inherit]` on base signals (`book/src/bridge/extern_rustqt.md`). Moc runs inside the build (`qt-build-utils` `tool/moc.rs`).

**`.ui`, QSS, resources, SVG, i18n.**
- `.ui`: no `uic` wrapper. `crates/qt-build-utils/src/tool/` has only `moc`, `rcc`, `qmlcachegen`, `qmltyperegistrar`, `qtpaths` [V]. `.ui` would need a manual `uic` call from `build.rs` (inference) or runtime `QUiLoader` (section 6).
- QSS: it is only a string passed to `setStyleSheet`, so it needs a bound `setStyleSheet` (hand-declared) [I].
- `.qrc`: `CxxQtBuilder::qrc(...)` and `qrc_resources(...)` run rcc [V].
- SVG: a `qt_module("Svg")` link is possible via the generic `qt_module` [I]; no Svg types are bound in `cxx-qt-lib` [V].
- QTranslator: not in the `cxx-qt-lib` list I read (`src/core/` not enumerated line by line) [U].

**Build story [V].**
- `qt-build-utils` finds Qt via `qmake` (`QMAKE` env or PATH; 0.9.1 note: "Fix finding qmake in PATH when only qmake6 is available"). 0.9.0 added "experimental Qt Minimal installation support" (feature `qt_minimal`, `book/src/getting-started/6-cargo-qt-minimal.md`).
- Cargo-only or CMake (Corrosion-style static lib). Book `getting-started/index.md`: tested on CI on Linux, Windows, macOS (x86_64); wasm32 compiled only. Windows with MSVC needs a matching runtime (`5-cmake-integration.md`, "Windows with MSVC").
- Static Qt 6 needs the `link_qt_object_files` feature (`4-cargo-executable.md`).
- MinGW vs MSVC mix: `qttypes` checks it; for cxx-qt see `platform.rs` [U for MinGW].

## 3. qmetaobject-rs (Woboq)

**Maintenance [V].** `crates.io`: qmetaobject 0.2.10 (2023-11-03), qttypes 0.2.12 (2024-10-22). Last repo commit `1f4d09a`, 2026-04-13, "chore: bump syn to 2". `CHANGELOG.md` "Unreleased" is empty. So: alive at maintenance level, no feature work visible, no release for about 3 years.

**Qt versions [V].** README: "Requires Qt >= 5.8". `qttypes/build.rs` rejects Qt < 5.6 and has Qt 6 branches (`>= 6.0.0`, `< 6.2.0` WebEngine note); `CHANGELOG.md` 0.2.12: "Prefer Qt6 over Qt5 if there is a qmake6 binary in PATH".

**Widgets coverage [V].** The README, "Comparison to Other Projects", says: "The primary goal of this crate is to provide idiomatic Rust bindings for QML. It focuses solely on QML, not QWidgets or any other non-graphical Qt API." `qttypes/src/lib.rs`: "this crate concentrate on the value types, not the widgets or the QObject". `qttypes/build.rs` links the Widgets library unconditionally, and `qmetaobject/src/qtdeclarative.rs` constructs a `QApplication` internally. That is the only Widgets use. No widget classes are wrapped.

**Subclassing [V].** README features: "Create object inheriting from QObject, QQuickItem, QAbstractListModel, QQmlExtensionPlugin, ...". Base classes are QObject family and QtQuick items. No QWidget base. Raw C++ is available through the `cpp!` macro (README "What if a wrapper for the Qt C++ API is missing?"), so a QWidget subclass can be written as inline C++ [I]. That is the shim route in disguise, without a normal C++ file layout.

**`.ui`, QSS, i18n [I].** Nothing in the crate for `.ui`; QML is the intended UI language, which the ticket excludes.

**Build story [V].** Cargo only, via the `cpp`/`cpp_build` crates (`qmetaobject/Cargo.toml`, `build.rs`) and `qttypes/build.rs` (qmake based, MSVC/MinGW mix check, macOS framework handling, Windows debug suffix).

**Verdict for this project [I].** Wrong tool: QML-first, Widgets deliberately out of scope.

## 4. rust-qt/ritual (qt_core, qt_gui, qt_widgets, qt_ui_tools)

**Maintenance [V].** Last commit in the repo `51b38ca` on 2021-05-18. `crates.io`: qt_widgets 0.5.0 published 2020-03-21 (the last release). Repo `qt_ritual/crate_templates/` has qt_core, qt_gui, qt_widgets, qt_ui_tools, qt_qml, qt_charts, qt_3d_*. Effectively unmaintained for 5 years; the docs.rs and rust-qt.github.io pages could not be fetched (section 8).

**Qt versions [V].** `qt_ritual_common/src/lib.rs`: `if qt_version_parsed.major != 5 { bail!("only Qt 5 is supported! ...") }`. **Qt 5 only.** No Qt 6.

**Widgets coverage [V].** This is the only route with generated wide Widgets coverage: the `qt_widgets` crate wraps QMainWindow, QPushButton, QSplitter, QSlider, QListWidget, and so on, generated from the Qt 5 C++ headers [I for the exact class list; docs.rs could not be fetched, but the generator targets the whole module]. `qt_ui_tools` provides `QUiLoader` plus `load_bytes` helpers (`qt_ritual/crate_templates/qt_ui_tools/src/impl_ui_loader.rs`) and the `#[ui_form("path.ui")]` macro (`qt_macros/src/lib.rs`). The `ui_form` doc says the struct's first field is `QBox<QWidget>` and each following field is a `QPtr<T>` named by `objectName`. QSS is a plain `setStyleSheet` call, which is generated. A `q_init_resource` macro supports `.qrc`.

**Subclassing [V + I].** The `qt_core` crate README says: "Some methods are missing, and some are inconvenient to use. Some methods are unsafe even though they are not marked as unsafe. Users must carefully track ownership of the objects". Searching `qt_ritual`, `qt_macros`, `cpp_core` for `paintEvent`, `eventFilter`, `event_filter` returned no hits [V]. The crates provide `#[slot]` (`qt_macros/src/lib.rs`) which wraps Rust closures as slot objects bound to a `QObject` parent, but no virtual-method override mechanism. [I] Overriding `paintEvent` or `mousePressEvent` is therefore not possible in pure ritual; the usual workaround is a C++ shim for each custom widget, which defeats the route.

**Build story [V/I].** Qt 5 via `qmake` (`qt_ritual_build`); platform details were not read beyond that [U]. Windows/macOS status on current OS and toolchain versions: not tested [U].

**Verdict [I].** Good widget coverage and a `.ui` loader, but dead, Qt 5 only, and no subclassing. Fails the paint/mouse-event requirement.

## 5. Thin C++ shim crate (own code on `cxx`, plus `cc` or `cxx-qt-build`)

This is a design, not a published project. Facts about the parts:

- `cxx` 1.0.205 published 2026-10-11 [V, crates.io]. `cpp` 0.5.11 published 2026-04-08 [V, crates.io].
- Custom widgets stay in C++ (the current `src/widgets/*.cpp`, about 8 small classes), exposing a small C-style or CXX bridge (create, show, set text, connect, emit callbacks into Rust) [I].
- The build can use the plain `cc` crate plus Qt tools, or `cxx-qt-build` as a build helper. `CxxQtBuilder::cpp_file` runs moc on C++ files (`crates/cxx-qt-build/src/lib.rs` ~347 [V]) and `qt_module("Widgets")` links Widgets [V]. `qt-build-utils` also exposes `moc()` and `rcc()` (`src/lib.rs` lines 278, 460) [V]. `uic` is not wrapped [V], so `build.rs` must call `uic` itself (or the shim checks in `ui_*.h` or uses `QUiLoader`).
- `uic` handles promoted widgets by emitting `#include` of the `<header>` listed in `<customwidgets>` (`qtbase/src/tools/uic/customwidgetsinfo.cpp` reads the section; header emission [I]). Original `.ui` files can be reused unchanged, with the C++ widget sources compiled into the same target.
- `QUiLoader` (`qttools/src/uitools/quiloader.h`) declares `virtual QWidget *createWidget(const QString &className, QWidget *parent, const QString &name)` [V]. Overriding it in a small C++ subclass lets the loader build the 8 promoted classes by name [I]. UiTools is a separate Qt module; its availability in packaged Qt builds on each OS is [U].
- Qt version in the shim is free: any Qt 5.15 or Qt 6 [I]. Qt 6.12.0 exists as a tag in `qt/qtbase` [V, `git ls-remote`].
- Cost: the app logic is Rust, but UI code stays about 2.7k LOC of C++ (ui + widgets). It is a hybrid, not a rewrite of the UI layer. This conflicts with a pure "Rust UI" goal; whether that goal is a requirement is a grilling question (ticket 07).

## 6. Comparison

| | cxx-qt | qmetaobject-rs | ritual (qt_widgets) | thin C++ shim |
|---|---|---|---|---|
| Qt 5 / 6 | 5.15 and all 6 [V] | >= 5.8, 6 partial [V] | 5 only [V] | any [I] |
| Widgets classes bound | none except QApplication [V] | none [V] | broad, generated, Qt 5 [V/I] | only what is needed [I] |
| Subclass + virtual override | yes for QObject-derived bases incl. QWidget [V docs, I for events] | QObject/QQuickItem; raw C++ via `cpp!` [V] | no [V/I] | native C++ [I] |
| `.ui` with promoted widgets | no uic; use `cpp_file` + manual uic or QUiLoader [V/I] | no | `QUiLoader` and `#[ui_form]` (no promoted widget support seen) [V/I] | uic or QUiLoader subclass [I] |
| QSS, SVG, qrc, i18n | qrc [V]; rest by hand-declaring [I] | QML only | QSS yes; qrc via macro [V/I] | all native [I] |
| Maintenance | active, API unstable [V] | idle, last release 2023-11 [V] | dead since 2021 [V] | n/a |
| Build | cargo or CMake, qmake based, CI on 3 OS [V] | cargo, qmake based [V] | cargo, Qt 5 qmake [V] | cargo + cc/cxx-qt-build [I] |

## 7. Findings (for the decision ticket)

1. **ritual** and **qmetaobject-rs** do not fit: ritual is Qt 5 only, dormant since 2021 and has no override mechanism; qmetaobject-rs states that QWidgets is out of scope.
2. **cxx-qt** is the only maintained route with a QObject-subclass code generator that accepts a QWidget-family base class. It ships no widget bindings, so every Qt Widgets class and every event type must be declared by hand. The two painted widgets, the mouse/key/drop overrides, `.ui` loading and QSS therefore all need hand-written bindings or C++.
3. **A thin shim** (C++ custom widgets + `uic`/`QUiLoader`, Rust for logic, built through `cxx-qt-build`'s `cpp_file`/`qt_module`) is the lowest-risk route to the exact UI; it is the only route where the full checklist is native Qt. The trade-off: about 2.7k LOC of UI stays C++.
4. Pure-Rust UI on cxx-qt is plausible but unproven for Widgets: no upstream example, and the hand-written binding surface is the cost. A small prototype (ticket 06) should test one painted custom widget (`SeekBar`) as `#[base = QSlider]` with `paintEvent`/`mouseMoveEvent` overrides and `QPainter(this)`.
5. Suggested split for the decision: cxx-qt plus selective C++ for widgets is a single build, so the choice between "all Rust UI" and "C++ widgets in a Rust app" can be made per widget later without changing toolchain.

## 8. What could not be verified

- docs.rs, rust-qt.github.io and doc.qt.io were unreachable (DNS blocked by the sandbox proxy). The ritual class list, `qt_widgets` API docs and ritual's stated platform support were therefore not read from published docs; the Qt API facts (`QUiLoader::createWidget`) come from the Qt source instead.
- GitHub API (open issues, archive state, stars, release pages) is blocked for this session, so issue-tracker evidence of known problems (for example "Widgets with cxx-qt", Qt 6 for ritual) is missing.
- Nothing was built or run. No Windows or macOS build was attempted. Claims about build behavior come from repo text only. Not tested: cxx-qt with a QWidget base that overrides `paintEvent`; `QPainter(this)` from Rust; Qt UiTools availability in packaged Qt on Windows/macOS; QtSvg and QTranslator bindings in cxx-qt-lib (not found in the files read, not exhaustively searched).
- Qt 5.15 end-of-support dates were not confirmed from a Qt source (this belongs to ticket 03).
- Licensing (LGPL vs commercial, static linking) was not researched; cxx-qt, qmetaobject-rs and ritual are permissively licensed per their repos (MIT/Apache in cxx-qt and ritual `README.md`; qmetaobject `LICENSE` not read) [V for the first two].
