# Research 02: libmpv render API inside a Qt widget, from Rust

Ticket: `.scratch/rust-rewrite/issues/02-research-libmpv-render-api-qt.md`
Researched: 2026-10-15. Sources fetched that day; versions below are those seen.

Labels: **[V]** verified in a primary source named in the line. **[I]** inference from verified facts. **[U]** not verified.

## 1. Short answer

1. The render API has OpenGL, D3D11 and software backends. There is no Vulkan or Metal backend in libmpv. [V: `render.h` on mpv master, lines 52-54; `DOCS/client-api-changes.rst`]
2. The D3D11 backend is new (client API 2.6, mpv 0.42.0). It is not in the headers that the Rust sys crates ship today (API 2.5). [V]
3. Of the Rust crates, only `libmpv2` (kohsine) is maintained and widely used. It exposes the OpenGL backend only. [V]
4. Qt Widgets can host the OpenGL render API with `QOpenGLWidget`. Qt 6.7+ `QRhiWidget` can use D3D11 on Windows and Metal on macOS, but libmpv cannot render into Metal. [V]
5. The current `wid` embedding is documented as working on X11, win32 and macOS only. It does not work on native Wayland. [V]

## 2. What `MpvHandler` and `OverlayHandler` use today

Read in `src/mpvhandler.cpp` and `src/overlayhandler.cpp` (this repo, commit `834a4b1`). [V]

| Use today | Where | Effect of render API |
|---|---|---|
| `mpv_set_option(mpv, "wid", INT64, &wid)`, with `wid = window->ui->mpvFrame->winId()` | `mpvhandler.cpp:30`, `bakaengine.cpp:19` | Removed. Replace with `mpv_render_context_create` on a GL (or D3D11) context owned by a Qt widget. |
| `input-cursor=no`, `cursor-autohide=no` | `mpvhandler.cpp:31-32` | `input-cursor` has no source with the render API: mpv gets no pointer events. Baka already handles mouse itself in `MainWindow::eventFilter`, so little changes. [V for mpv side, I for Baka side] |
| Property observation, `mpv_wait_event`, wakeup callback posting `QEvent::User` | `mpvhandler.cpp:37-47,139` | Unchanged. Event loop is separate from rendering. |
| `mpv_command` / async commands, `loadfile`, `show_text` (commented out), `osd-msg seek` | `mpvhandler.cpp:437,449,664-675,920-925` | Unchanged. `osd-msg seek` still draws mpv's own OSD, which the render API draws into your FBO. [I] |
| `overlay_add` / `overlay_remove` with a raw BGRA pointer (`"&"+address`) | `mpvhandler.cpp:250-272`, called from `overlayhandler.cpp:~125` | Command still exists (`input.rst`, `overlay-add`, ids 0-63). The ids 62 and 63 that Baka uses are in range. [V] |
| Duplicate QLabel over `mpvFrame` showing the same QImage | `overlayhandler.cpp` (label created as child of `mpvFrame`) | This is a workaround for `wid`: a native child window hides Qt widgets drawn under it. See section 5. |
| Properties `current-vo`, `hwdec-active`, `osd-width/height`, `avsync`, `estimated-vf-fps`, track and chapter lists | `mpvhandler.cpp:71-77,850` | Unchanged. `current-vo` will read `libmpv`. |
| `vo` option rewrite for "interpolation": appends `:interpolation` to the VO string | `mpvhandler.cpp:633-655` | Likely breaks. The VO becomes `libmpv`, and appending `:interpolation` to a VO string probably no longer works. Set the `interpolation` option directly. [I: not checked against `options.rst` history; verify in the port] |
| No `hwdec` set anywhere | grep of `mpvhandler.cpp` | Hardware decoding is off today (mpv default). Enabling it becomes an explicit choice with the constraints in section 6. [V: `options.rst` says hwdec is not enabled by default] |

Adjacent: `src/platform/linux.cpp` uses `QX11Info` (Qt5 `x11extras`) for the compositor check and `SetAlwaysOnTop`. Those are X11-only and out of scope here, but they matter for Wayland. [V]

## 3. The render API itself

All from mpv master headers (`include/mpv/client.h`, `render.h`, `render_gl.h`, `render_d3d11.h`), client API version 2.6. [V]

- Create the render context **before** playback starts. Without it, video init fails or falls back to a VO that opens its own window. (`render.h`, "Context and handle lifecycle".)
- Create with `mpv_render_context_create` using `MPV_RENDER_PARAM_API_TYPE` plus backend params. Set the `vo` option to `libmpv` as the `libmpv2` example does. Pass `MPV_RENDER_PARAM_ADVANCED_CONTROL`: the header says it is "strongly recommended" but you must then call `mpv_render_context_update` and never block the render thread on libmpv calls.
- One render context per mpv core. Free it before `mpv_terminate_destroy`, or behavior is undefined. Baka's destructor currently only calls `mpv_terminate_destroy`; it needs the free first.
- Threading: all `mpv_render_*` calls are serialized, never from the wakeup or update callbacks, and with OpenGL the same GL context must be current. Do not call other libmpv functions from the render thread.
- Draw with `mpv_render_context_render` into an FBO (`MPV_RENDER_PARAM_OPENGL_FBO`; `fbo` 0 means the default framebuffer). The header lists GL state mpv leaves changed: viewport, scissor, blend func, clear color. Qt code that shares the context must reset these.
- `mpv_render_context_set_update_callback` tells you a new frame is ready. It must only schedule work, not render.
- Input: the render API gives no mouse or keyboard input. You send it with `mouse`, `keypress`, `keydown`, `keyup` commands. (`mpv-examples/libmpv/README.md`; Baka mostly handles input itself.)
- OpenGL: mpv does not load the system GL library. It uses the `get_proc_address` callback you give it.
- Software backend exists but the header calls it "extremely simple (but slow)" and says you probably do not want it.

## 4. Rust crates

Data from the crates.io API and the published `.crate` tarballs, fetched 2026-10-15. GitHub API access was blocked in this session, so repo activity (stars, open issues, last push) is **[U]**.

| Crate | Version / date | License | Bundled client API | Render backends exposed | Notes |
|---|---|---|---|---|---|
| `libmpv2` | 6.0.0, 2026-05-12, ~98k downloads, 11 reverse deps | LGPL-2.1 | via `libmpv2-sys` 4.0.1 (headers: client 2.5) | OpenGL only (`RenderParamApiType` has one variant, `OpenGl`). Params include X11 and Wayland display, advanced control, next-frame info, `report_swap`. No SW, no D3D11. | Edition 2024. 6.0.0 changelog fixes use-after-free races in the render update callback and wakeup callback, and makes `RenderContext` own a reference to `Mpv`. Repo: `kohsine/libmpv-rs`. Example `examples/opengl.rs` uses SDL2. |
| `libmpv2-sys` | 4.0.1, 2025-12-29 | (not checked) | client 2.5; headers `client.h`, `render.h`, `render_gl.h`, `stream_cb.h` | OpenGL, SW constants exist in bindings; no D3D11 | bindgen at build time, pregenerated fallback. Pinning a newer `render.h` for D3D11 means regenerating. [I] |
| `libmpv`, `libmpv-sys` | 2.0.1 (2020-09), 3.1.0 (2020-07) | | old | OpenGL (old API era) | Unmaintained since 2020. Repo `ParadoxSpiral/libmpv-rs` (the upstream of `libmpv2`). Do not use. |
| `libmpv-sirno`, `libmpv-sys-sirno` | 2.0.x fork, 2022-12 | | old | | Fork of the 2020 crates. Stale. |
| `mpv` | 0.2.3, 2017 | | pre-render-API | none | Dead. |
| `rsmpv` + `rsmpv-sys` | 0.2.1, created 2026-09-04, ~800 downloads | MIT OR Apache-2.0 (sys: ISC) | hand-written, client 2.5 | OpenGL and software (`render_opengl`, `render_software`, `new_software`) | Very new, one maintainer, no reverse deps seen. Claims a clean-room build from the ISC headers to avoid LGPL on the wrapper. Repo `ophymx/rsmpv`. The license angle matters only if the project license needs it; the wrapper's license is separate from the libmpv you link (GPLv2+ by default, LGPL if built with `-Dgpl=false`). |
| `mpv-client`, `mpv-client-sys` | 1.1.0 / 1.0.1 | | | n/a | For writing mpv **plugins**, not hosts. Wrong tool. |
| `tauri-plugin-libmpv`, `tauri-plugin-mpv` | 0.3.x / 0.5.x | | | n/a | Tauri-specific. Not relevant to Qt. |

[I] Recommendation basis: `libmpv2` is the only maintained, used option with the render API and recent safety fixes. If D3D11 is wanted, raw FFI against current headers (or a small `-sys` update) is needed, because no crate wraps `MPV_RENDER_API_TYPE_D3D11`. A thin hand-written FFI layer for the render part (about 10 functions) is also a viable alternative to taking a dependency for it.

## 5. Putting the render API inside a Qt widget

### 5.1 OpenGL path (works on all three OSes)

- `QOpenGLWidget` (Qt 5.4+, still present in Qt 6 `QtOpenGLWidgets`) gives a GL context and an FBO. Call `mpv_render_context_create` in `initializeGL`, `mpv_render_context_render` in `paintGL` with `QOpenGLWidget::defaultFramebufferObject()`. [V for the widget; I for the mpv wiring, which follows `render_gl.h` and the `libmpv2` SDL example]
- Overlap behavior, from Qt's own docs (`qopenglwidget.cpp`): widgets **on top of** a QOpenGLWidget work as expected. Widgets underneath a transparent one do not. Adding one turns on OpenGL-based composition for the whole top-level window, and the rest of the widgets still paint with the software painter. [V]
- This is the fit for Baka: the on-screen controls and the existing QLabel/QSS overlays sit above `mpvFrame`. With the QOpenGLWidget, the plain Qt widget overlays work and the duplicate `overlay_add` bitmap is no longer needed for visibility. [I]
- The Qt docs name an alternative: `QOpenGLWindow` inside `QWidget::createWindowContainer()`. It is faster (no extra composition) but has "overlaps, transparency" limits and the docs say to use it only if there is no other choice. [V] That brings back the native-window overlay problem, so it is not a good fit. [I]
- `get_proc_address`: use `QOpenGLContext::getProcAddress`. [I]
- Threading: simplest is render on the GUI thread inside `paintGL`, triggered by the update callback posting `update()` to the GUI thread. [I] A separate render thread needs a `QOpenGLContext` moved to that thread, and mpv's own warning about not blocking on libmpv calls applies. Not needed for a first version. [I]

### 5.2 Per-OS notes for the OpenGL path

- **Linux, X11 and Wayland.** For hardware decoding with Intel GPUs, `render_gl.h` says EGL is required and you must pass the native display: `MPV_RENDER_PARAM_X11_DISPLAY` or `MPV_RENDER_PARAM_WL_DISPLAY`. [V] Qt 6 exposes both through `QNativeInterface::QX11Application::display()` and `QNativeInterface::QWaylandApplication::display()` (seen in `qguiapplication_platform.h`). [V] Qt 5 used `QX11Info::display()`, which Baka already uses. [V] The Qt5 Wayland display accessor: **[U]**.
- **Windows.** `render_gl.h`: "ANGLE is required" for hardware decoding via OpenGL ("although in theory GL/DX interop could be used"). [V] Whether Qt 6 ships ANGLE and whether a Qt desktop-GL context can do d3d11va interop: **[U]**. This is the weak point of the OpenGL path on Windows. The new D3D11 backend (5.3) is the documented fix.
- **macOS.** `render_gl.h`: CGL required for hwdec (`CGLGetCurrentContext()` non-NULL). [V] The `mpv-examples` README says window embedding "seems to have stability problems on OSX when using the Qt toolkit" and recommends the render API on OSX. [V] Apple's OpenGL deprecation status and its practical effect: **[U]** (Apple docs not fetched; not a primary source I read).

### 5.3 Vulkan, Metal, D3D11

- libmpv has **no** Vulkan or Metal render backend. [V: `render.h`] `QRhiWidget` (Qt 6.7+) can run on Metal, Vulkan, D3D11/12 and OpenGL, with defaults of Metal on macOS, D3D11 on Windows, OpenGL elsewhere. [V: `qrhiwidget.cpp`] But libmpv cannot render into a Metal or Vulkan texture, so those APIs are not usable for mpv video. [I from the two facts above]
- D3D11: `MPV_RENDER_API_TYPE_D3D11` takes an `ID3D11Device*` (`mpv_d3d11_init_params`) and renders into an `ID3D11Texture2D*` (`mpv_d3d11_fbo`) created on the **same device**, bind flag render target, not MSAA, not sRGB, formats include `R16G16B16A16_FLOAT` and `R10G10B10A2_UNORM`. [V: `render_d3d11.h`] Qt's `QRhiWidget` on Windows defaults to D3D11, and `QRhi` has `QRhiD3D11NativeHandles` and `QRhiTexture::createFrom` to wrap native objects. [V: `qrhiwidget.cpp`, `qrhi.cpp`] So a Qt 6.7+ `QRhiWidget` on D3D11 could share its device with mpv. Nobody has been shown doing this; **[U]** whether it works in practice, and QRhi classes carry "no source or binary compatibility guarantees" across Qt releases. [V]
- `QRhiWidget` caveat: one QRhi API per top-level window. Mixing a GL widget and a D3D11 widget in one window will break one of them. [V]
- The D3D11 backend's release status: it appears on mpv master and in `client-api-changes.rst` under "mpv 0.42.0". Whether a Windows libmpv build with it is available to the project: **[U]**.

## 6. Hardware decoding and HDR per OS

Hardware decoding, from `DOCS/man/options.rst` (`--hwdec`) and `render_gl.h`. [V]

| OS | Direct hwdec choices that need a GPU API | Copy-back (works with any GL setup) | Render-API constraint |
|---|---|---|---|
| Windows | `d3d11va` (needs `gpu`/`gpu-next` with `d3d11` or `angle` context; Windows 8+) | `d3d11va-copy` | libmpv + OpenGL needs ANGLE for direct decode. With the D3D11 backend, `d3d11va` runs on the device you pass; a device made with `D3D11_CREATE_DEVICE_SINGLETHREADED` allows only copy-back. |
| Linux | `vaapi` (Intel, AMD), `nvdec`, `drm`, `vulkan` | `vaapi-copy`, `nvdec-copy`, `drm-copy` | Intel: EGL plus native display param. NVIDIA: GLX or EGL. `--vo=vaapi` and `dmabuf-wayland` listed in the hwdec docs are standalone VOs, so they do not apply to the render API. [I] |
| macOS | `videotoolbox` (needs `gpu`/`gpu-next` on macOS, or `--vo=libmpv` on iOS) | `videotoolbox-copy` | Needs a CGL context. The doc text lists `libmpv` only for iOS; macOS uses the "cocoa-cb" path only inside mpv. Whether VideoToolbox zero-copy works through the render API on macOS: **[U]**. |

- `auto-copy` is the safe baseline on every OS: it only picks copy modes, "works with all video filters and VOs". [V]
- `options.rst` says only `vaapi`, `nvdec`, `cuda` and `vulkan` hwdecs work with a Vulkan GPU backend; most others need OpenGL. [V] (Relevant only to mpv's own VOs.)
- Direct rendering (`vd-lavc-dr`): "The libmpv render API has optional support." [V]

HDR:

- HDR passthrough in mpv is driven by `--target-colorspace-hint`, which "requires a supporting driver and `--vo=gpu-next`" and is auto-supported on Wayland, D3D11 and winvk contexts. [V: `options.rst`] That option sets metadata on the swapchain that mpv owns.
- With the render API the host app owns the framebuffer and swapchain. [V: `render.h`] So the metadata cannot be set by mpv. [I] HDR to an HDR display would need Qt (or the app) to configure an HDR swapchain and pass a matching float or 10-bit target. Whether Qt Widgets can present an HDR swapchain: **[U]**. HDR tone-mapping to SDR (convert to an SDR target) works in the shader path regardless. [I, from `options.rst` text that says you can set `--target-trc` to an SDR function so "any input will be converted to SDR"; behavior through libmpv not tested]
- `render.h` has `MPV_RENDER_PARAM_ICC_PROFILE` for display color management. [V] `MPV_RENDER_PARAM_AMBIENT_LIGHT` is deprecated since API 2.5. [V]
- Baka's current `wid` path gets mpv's native window and could reach `gpu-next`'s swapchain path. So HDR passthrough is a **regression risk** moving to the render API. [I] Whether the project needs HDR at all: decision for the ticket 05 / 04 interviews, not a research fact.

## 7. What has to change (summary list)

1. Replace the `wid` option with a render context created before playback, on a `QOpenGLWidget` (first choice) in `mpvFrame`'s place. Free the render context before `mpv_terminate_destroy`.
2. Set `vo=libmpv`; remove the `vo` string rewrite for interpolation and set mpv's `interpolation` option directly. [I]
3. Add `MPV_RENDER_PARAM_ADVANCED_CONTROL`, the update callback (schedule `update()` on the GUI thread), and `mpv_render_context_update` handling.
4. Choose a hwdec policy. Today there is none. Default candidate: `auto-copy` for portability, with `auto` enabled only where the GL setup meets the table in section 6.
5. Pass the X11 or Wayland display (from Qt) for EGL-based Linux hwdec.
6. Drop the duplicate QLabel overlay trick. Keep either the Qt-drawn overlay widgets above the GL widget or mpv's `osd-overlay` (ASS text, section `input.rst`); `overlay-add` still works but is documented as a bitmap path. [V] The Qt-drawn approach keeps the exact look from `overlayhandler.cpp` (QFont, QPainterPath outline, QColor), which the hard requirement for exact UI favors. [I]
7. Input: no `input-cursor`/`wid` focus behavior. Forward mouse and key events from Qt to mpv only if mpv bindings are wanted; Baka mostly handles input in Qt already. [I]
8. Linux X11-only helpers (`QX11Info`, `SetAlwaysOnTop` in `src/platform/linux.cpp`) need Wayland-safe replacements. Separate topic. [I]

## 8. Could not verify

- GitHub repo state for `kohsine/libmpv-rs` and `ophymx/rsmpv` (last push, open issues, stars): the proxy blocked the GitHub API for this session.
- `doc.qt.io` was blocked by the proxy. Qt facts here come from Qt's own documentation source comments in `qt/qtbase` (`qopenglwidget.cpp`, `qrhiwidget.cpp`, `qrhi.cpp`, `qwidget.cpp`, `qwindow.cpp`, `qguiapplication_platform.h`) on the `dev` branch, not from a released version's doc page. Behavior in Qt 5.15 specifically was not checked.
- Whether Qt 6 desktop GL on Windows can do d3d11va interop, and whether Qt still ships ANGLE.
- Whether `QRhiWidget` and `mpv_render_context` can share a D3D11 device in practice, and whether mpv 0.42.0 with the D3D11 backend has a Windows build the project can ship.
- Zero-copy VideoToolbox through the render API on macOS, and Apple's OpenGL deprecation status.
- HDR passthrough through Qt Widgets on any OS.
- The mpv `interpolation` option name change relative to the `:interpolation` VO suffix Baka uses (history of `options.rst` not read).
- Nothing was run. No prototype or build was attempted; ticket 06 covers that.

## 9. Sources

- mpv headers on `master`: `include/mpv/client.h`, `render.h`, `render_gl.h`, `render_d3d11.h` (raw.githubusercontent.com/mpv-player/mpv).
- mpv docs on `master`: `DOCS/man/options.rst`, `vo.rst`, `input.rst`, `DOCS/client-api-changes.rst`.
- `mpv-player/mpv-examples`: `libmpv/README.md`.
- crates.io API and tarballs: `libmpv2` 6.0.0 (`CHANGELOG.md`, `Cargo.toml`, `src/mpv/render.rs`, `examples/opengl.rs`), `libmpv2-sys` 4.0.1 (`include/`), `rsmpv` 0.2.1 (`README.md`, `src/render.rs`).
- Qt source and doc comments on `dev`: `qtbase/src/openglwidgets/qopenglwidget.cpp`, `src/widgets/kernel/qrhiwidget.cpp`, `src/gui/rhi/qrhi.cpp`, `src/widgets/kernel/qwidget.cpp`, `src/gui/kernel/qwindow.cpp`, `src/gui/kernel/qguiapplication_platform.h`.
- This repo: `src/mpvhandler.cpp`, `src/overlayhandler.cpp`, `src/bakaengine.cpp`, `src/platform/linux.cpp`.
