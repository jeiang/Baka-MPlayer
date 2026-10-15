# Research 03: Qt 6 vs Qt 5 and cross-platform packaging

Ticket: `.scratch/rust-rewrite/issues/03-research-qt-version-packaging.md`
Date: 2026-10-15
Method: primary sources only. `doc.qt.io`, `qt.io`, `learn.microsoft.com`, `mpv.io`, `gnu.org`, `docs.flatpak.org`, `docs.appimage.org` were blocked by the sandbox proxy (403). Qt docs were read from their source (`.qdoc` files and headers in the `qt/*` GitHub repos via `raw.githubusercontent.com`). Apple docs were read through `developer.apple.com` JSON. Microsoft docs were read from the `MicrosoftDocs/*` repos.
Labels: **[V]** verified in a source cited here. **[I]** inference, not stated by a source.

## 1. Short answer

1. Qt 6 Widgets can do everything this app needs except two Qt5 modules: `QtWinExtras` and `QtX11Extras`. Both are removed from Qt 6. [V]
2. Both gaps are small here: one Windows taskbar thumbnail toolbar (3 buttons) and a few Xlib calls that need `QX11Info`. [V from code, fix is I]
3. The repo is GPL, so Qt can be linked under GPL or LGPL, dynamic or static. Static is allowed. Dynamic is still the simpler path. [V for license terms, I for the recommendation]
4. Per-OS shipping tools exist for Qt 6: `windeployqt`, `macdeployqt`, `linuxdeploy-plugin-qt` (AppImage), and the KDE Flatpak runtime. [V]
5. Signing is needed to avoid friction on macOS (notarization is required for Developer ID software) and is optional but advisable on Windows. [V for macOS, partly V for Windows]

## 2. What the repo uses that is Qt5-only

Grounded in `src/Baka-MPlayer.pro` and `src/platform/*`.

| Item | Where | Qt 6 status |
|---|---|---|
| `QT += winextras` (win32 only) | `Baka-MPlayer.pro:38` | Module removed. Its README says: "The QtWinExtras module has been deprecated and is no longer part of Qt" and that functions were "incorporated into other parts of Qt, or in some cases removed". [V] `qt/qtwinextras` `dev` README, QTBUG-89564 |
| `QWinThumbnailToolBar`, `QWinThumbnailToolButton` | `ui/mainwindow.h:18-19,91-92`, `ui/mainwindow.cpp:857-891` (Win only, prev / play-pause / next buttons) | Part of QtWinExtras, so gone. [V] The Windows API underneath is `ITaskbarList3::ThumbBarAddButtons`, max 7 buttons, needs the `TaskbarButtonCreated` window message first. [V] `MicrosoftDocs/win32` `shell/taskbar-extensions.md`. Replacement is a direct Win32 call from our own code (inference: [I]). I did not find a Qt 6 class for it. |
| `QT += x11extras`, `PKGCONFIG += x11` (unix, not mac) | `Baka-MPlayer.pro:29-30` | Module removed. Same README text as above, QTBUG-93633. [V] |
| `QX11Info::appScreen()`, `QX11Info::display()` | `platform/linux.cpp` (`DimLightsSupported`, `SetAlwaysOnTop`) | Qt 6 has `QNativeInterface::QX11Application` with `display()` and `connection()` (xcb). [V] `qtbase` `qguiapplication_platform.h`, 6.8 and dev. It has no `appScreen()`. Use `XDefaultScreen(display)` instead ([I]). |
| `QRegExp` | `platform/*.cpp`, `util.cpp:12` | Not in Qt 6 core. Exists only in the `qt5compat` module (`core5/text/qregexp.h`). [V] Use `QRegularExpression` ([I]). |
| `QDesktopWidget`, `qApp->desktop()` | `ui/mainwindow.cpp:7`, `bakacommands.cpp:5,436`, `gesturehandler.cpp:4,38-39`, `widgets/dimdialog.cpp:9,41` | Header `widgets/kernel/qdesktopwidget.h` is absent in 6.8 (404). [V] `QScreen` is in `gui/kernel/qscreen.h`. [V] Replacement is `QScreen` ([I]). |
| mpv embedding by `wid` | `mpvhandler.cpp:30` (`mpv_set_option "wid"`) | Not a Qt version issue. Out of scope here (ticket for the mpv integration). [I] |

Other platform code is plain Win32 (`SetWindowPos`) or `QDesktopServices` and does not depend on Qt version. macOS shim (`platform/osx.cpp`) stubs `SetAlwaysOnTop` and `DimLightsSupported`. [V]

Also in the `.pro`: `WINVER=0x0501` (WinXP), a 32-bit variant (`win32.cpp`), `c++11`. None of these survive Qt 6 (see section 3).

Note: if the rewrite uses Qt from Rust, the Rust binding decides which Qt 6 APIs are reachable. That is outside this ticket.

## 3. Qt 6 platform floor (shapes "drop legacy targets")

From `qt/qtdoc` `supported-platforms.qdocinc` and `qt/qtbase` `.cmake.conf`:

- Windows: Windows 10 (1809 or later), Windows 11, x86_64 and ARM64. Compilers in the 6.8 branch: MSVC 2022, Mingw-w64 13.1. In `dev`: MSVC 2026/2022, Mingw-w64 15.1. "Qt 6.12 will be the last version to support Windows 10." No 32-bit Windows row. [V]
- macOS: branch 6.8 (6.8.4): target macOS 12 or higher, Xcode 15, SDK 14; arch `x86_64`, `x86_64h`, `arm64`. Branch 6.10 (6.10.3): macOS 13 or higher, tested up to 26. [V]
- Linux: tested distros listed include RHEL 9/10, Ubuntu 22.04/24.04 (x86_64, arm64), Debian 11.6/12, openSUSE 16, SLES 15. Official Qt binaries are linked against glibc 2.34. [V]
- Qt's support statement: "Qt may still run on unsupported platforms", but that is not support. [V]

Version choice: I read branches 6.8, 6.10, 6.11 and `dev` (6.13.0). I did not verify which are LTS or the current release list. Qt 5.15 end of life is stated in the ticket but I could not source it (qt.io blocked). [not verified]

## 4. Packaging per OS

### Windows
- `windeployqt` copies Qt DLLs and plugins next to the exe. Options seen in source: `--dir`, `--libdir`, `--plugindir`, `--no-translations`, `--compiler-runtime` / `--no-compiler-runtime`, `--no-opengl-sw`, `--no-system-d3d-compiler`, `--no-ffmpeg`, `--force-openssl`. [V] `qtbase` `windeployqt/main.cpp` (6.8). It can also bundle the MSVC runtime installer (`vc_redist`). [V, code at lines ~1535-1545 in `dev`]
- libmpv is not a Qt dependency, so `windeployqt` will not copy it. Put `libmpv-2.dll` (or equivalent) beside the exe by a build step. [I]
- Signing: `signtool sign` with `/fd` (file digest) and `/tr` + `/td` (RFC 3161 timestamp). Without `/tr` the file is not timestamped. [V] `MicrosoftDocs/windows-driver-docs` `signtool.md`. Microsoft also has a managed service called Artifact Signing (earlier "Trusted Signing"). [V] `MicrosoftDocs/azure-docs` `artifact-signing/overview.md`. I did not read its eligibility rules, price, or whether a hobby project can enroll. [not verified]
- The old update path (`Util::DownloadFileUrl()` zip URLs in `win32.cpp`/`win64.cpp`) is tied to the 32/64 split and to `bakamplayer.u8sand.net`. [V from code] Fate of the updater is another ticket.

### macOS
- `macdeployqt <app>.app` copies Qt frameworks and plugins into the bundle. Flags in source: `-dmg`, `-no-strip`, `-libpath`, `-executable`, `-codesign=<ident>` (default is ad-hoc), `-no-codesign`, `-hardened-runtime`, `-timestamp`, `-sign-for-notarization=<ident>`, `-appstore-compliant`, `-fs`. [V] `qtbase` `macdeployqt/main.cpp` (dev).
- Notarization (Apple): Developer ID software built after 2019-06-01 must be notarized to run under macOS 10.15 and later. [V] Apple "Notarizing macOS software before distribution". Requirements listed: code-sign every executable, use a "Developer ID" certificate (not ad hoc), Hardened Runtime on, secure timestamp, no `com.apple.security.get-task-allow` entitlement, SDK 10.9 or later. [V] Same page. Accepted containers: app in ZIP, UDIF disk image, signed flat pkg. [V] "Customizing the notarization workflow". The page says the old `altool` upload stopped working 2023-11-01 and the replacement is `notarytool` ([V], the command names were stripped from the JSON I read; naming `notarytool` is [I] from my own knowledge).
- Needs an Apple Developer Program account for the Developer ID certificate. [I: the Apple pages I read assume it but I did not find the fee or enrollment page.]
- libmpv must be put in `Contents/Frameworks` and its install names rewritten; every nested dylib must be signed. [I] `macdeployqt -libpath` helps find dylibs. [V flag exists]
- Arch: Qt supports `arm64` and `x86_64`. Universal builds are not covered in what I read. [not verified]

### Linux
- AppImage: `linuxdeploy` plus `linuxdeploy-plugin-qt`. The plugin says it supports Qt 5 and 6 and bundles Qt plugins and translations; Wayland needs extra env vars (`waylandcompositor` / `libqwayland-*`). [V] `linuxdeploy-plugin-qt` README. AppImage is a one-file format that runs across distros without install. [V] `AppImage/AppImageKit` README. glibc floor: build on the oldest distro you want to support. [I]
- Flatpak: `org.kde.Platform` / `org.kde.Sdk` runtime "will offer us most Qt modules and KDE Frameworks". Example uses `--socket=fallback-x11 --socket=wayland --device=dri`. [V] `flatpak/flatpak-docs` `qt.rst`. The doc example still shows runtime-version `5.15-24.08`, so I could not confirm from it which KDE runtime branch carries Qt 6. [not verified]
- Flatpak is sandboxed. This app uses `XSendEvent` to set always-on-top and an X11 atom hack for compositor detection; both only work on X11 (code already skips Wayland). [V from code] The mpv `wid` embedding also depends on X11 (inference [I], other ticket).
- Distro packages (`debian/`, `arch/`) use system Qt and system libmpv. No bundling needed. [V that dirs exist; contents not audited]

## 5. License and linking obligations

Project license: `LICENSE` is GPL v2 text. `debian/copyright` says `GPL-2.0+` (v2 or any later). `aboutdialog.ui:300` says "either version 2 of the License, or (at your option) any later version". [V]

Qt: the sources of Qt Widgets (`qwidget.cpp`), Qt Gui (`qguiapplication.cpp`), Qt Svg (`qsvgrenderer.cpp`) and Qt5Compat (`qregexp.h`) carry `SPDX: LicenseRef-Qt-Commercial OR LGPL-3.0-only OR GPL-2.0-only OR GPL-3.0-only`. [V] So Qt can be taken under GPL-2.0-only, which matches this project directly. Qt's docs on deployment say static linking gives one executable but "you cannot deploy plugins", and warn that third-party code inside Qt "is not licensed using the same dual-license model", so check those when linking statically. [V] `qtdoc` `deployment.qdoc`.

What follows:
- Because the app is GPL, the project can simply use Qt under GPL. Source must be offered to anyone who gets a binary. That already holds for this repo. [I from GPL, which I did not re-read on gnu.org (blocked)]
- LGPL-3.0 is also available. LGPL-3.0 text is in `qtbase/LICENSES`; combining a GPL-2.0-or-later app with LGPL-3.0 Qt means the result is under GPL-3.0 terms. [I] Not needed unless someone wants a non-GPL fork.
- Dynamic vs static: dynamic keeps LGPL "relink" obligations trivial and matches `windeployqt` / `macdeployqt` / AppImage flows. Static is legal under GPL, but loses plugins (Qt's own note above) and makes the build harder. Recommendation: dynamic. [I]
- Qt Commercial would be needed only to close the source. Not relevant now.
- libmpv: `mpv-player/mpv` `Copyright` says mpv is GPLv2+ by default, and LGPLv2.1+ only when built with `-Dgpl=false`; the file warns `-Dgpl=false` "does not in itself create a LGPLv2.1+ license grant". [V] `meson.build` lists `license: ['GPL2+', 'LGPL2.1+']`. For this GPL app either build is fine. A GPL libmpv would block any future permissive relicensing. [I]
- Not checked: licenses of FFmpeg/libass and other libs inside a bundled libmpv; Rust crate licenses; font and icon licenses (`etc/font/LICENSE` exists). [not verified]

## 6. Effects on the map

- Qt 6 is the only Qt target to consider; Qt 5 is dropped. [I from section 2 and 3]
- Two code areas must be redone for Qt 6 regardless of language: the Windows taskbar toolbar and Linux X11 helpers. Both are small. [V counts, I effort]
- Packaging tool per OS is known and does not depend on language, except that a Rust build must invoke these deploy tools after `cargo build`. [I]
- Windows 10 1809 and macOS 12/13 are the floors. This drops WinXP, 32-bit, and old macOS. [V]

## 7. Not verified

- Qt 5 EOL date and current LTS list (qt.io and doc.qt.io blocked).
- Qt 6 deployment pages for Linux, Windows and macOS (read the tool source instead of the prose).
- Any statement in the Qt Qt-for-Linux docs on the minimum Wayland / xcb setup for Qt 6.
- Whether Windows Artifact Signing or an EV certificate is open to this project; cost of Apple Developer ID.
- GPLv2 vs LGPLv3 compatibility text (gnu.org blocked). Section 5 conclusions about GPL combination are my inference.
- Which KDE Flatpak runtime branch carries which Qt 6 minor version.
- Whether a Qt 6 class replaces the thumbnail toolbar (I found none, but only searched `qtbase` file names reachable by raw URL, not the full tree).
- Universal (arm64 + x86_64) macOS builds and libmpv bundling steps (mpv's own build/packaging docs not read).
- All Qt facts are for branches 6.8, 6.10, 6.11 and `dev` as of 2026-10-15; `dev` content may change.
