# Research 11: libmpv bundling, signing, and licensing per OS

Issue: jeiang/Baka-MPlayer#11. Date of research: 2026-10-15.
Route (locked in #8): Qt 6, cxx-qt, bundled libmpv through the `libmpv2` crate and the OpenGL render API, project license GPL-2.0-or-later.
Earlier work: #4 (could not verify signing, Flatpak, universal macOS, libmpv bundling, dependency licenses because docs sites were blocked).

Marking: **(V)** = verified in a primary source read in this session, with the source named. **(I)** = inference, my reasoning on top of verified facts. Section 9 lists what I could not verify.

How sources were reached: the sandbox proxy blocks most documentation sites (mpv.io, doc.qt.io, learn.microsoft.com, signpath.org, gnu.org, docs.flathub.org, invent.kde.org). Reachable were `raw.githubusercontent.com`, anonymous `git clone` of public GitHub repos, `gitlab.com` raw files, `developer.apple.com` (including its `tutorials/data/...json` documentation endpoints), and `apache.org`. All source files below were read at their default branch on 2026-10-15. Branch tips move; re-check before relying on a version.

---

## 1. Summary

| Topic | Result |
|---|---|
| Windows libmpv | mpv upstream CI builds `libmpv-2.dll` (shared libmpv, static dependencies) in GPL and LGPL variants, but only as "development builds", not releases (V). Community builds exist (zhongfly, shinchiro). For a pinned, reproducible release, build it ourselves from `ci/build-mingw64-full.sh` (I). |
| macOS libmpv | mpv upstream publishes `mpv.app` tarballs per architecture, no universal build and no standalone libmpv dylib set (V). Universal libmpv dylibs exist from media-kit and IINA (V that they exist, not how they are made). Own build: two arch builds plus `lipo` (I). |
| Linux libmpv | Flatpak: build FFmpeg and mpv as modules, as `io.mpv.Mpv` does (V). The KDE runtime recipes and freedesktop-sdk master do not ship libmpv (V). AppImage: bundle libmpv and its non-system libraries with linuxdeploy; the AppImage exclude list keeps GL, EGL, gbm, drm, wayland-client, ALSA, fontconfig, freetype, harfbuzz, zlib on the host (V). |
| License effect | A default GPL libmpv build is GPLv2+. The official Windows GPL libmpv is a GPLv3 combination because FFmpeg is built with `--enable-gpl --enable-version3` (V). Expect the distributed binary to be GPLv3-level, which GPL-2.0-or-later allows (I). Source offer and notices are required (section 4). |
| Windows signing | Free route for open source: SignPath Foundation "Open Source Code Signing" (V that the edition exists; eligibility rules not verified). Paid route: Microsoft Artifact Signing (formerly Trusted Signing), paid Azure subscription, individuals only in the US and Canada (V). Microsoft's example uses an RFC 3161 timestamp (V). |
| macOS signing | Apple Developer Program, 99 USD per year, fee waiver only for nonprofits, schools, and government (V). Developer ID plus Hardened Runtime plus secure timestamp plus notarization with `notarytool` (V). |
| Flatpak runtime | KDE runtime branch 6.10 = Qt 6.10.3, 6.11 = Qt 6.11.2, 6.9 = 6.9.3, 6.8 = 6.8.3 (V from recipes at branch tips). No libmpv in any of them (V). |
| Windows hardware decode | Qt 6 dropped ANGLE (V). mpv's `d3d11va` zero-copy path for the OpenGL render API needs an ANGLE EGL context (V). With Qt 6 desktop OpenGL on Windows only copy-back decode (`d3d11va-copy`) is available through the GL render API (I from V). libmpv now has a native D3D11 render API (V), which is the route to zero-copy on Windows. |

---

## 2. How others bundle libmpv

### 2.1 Windows

- **mpv upstream CI** (V). `ci/build-mingw64-full.sh` builds "static mpv and libmpv builds on top of the FFmpeg-Builds container images". FFmpeg and its dependencies are static libraries; mpv is built with `--default-library=shared --prefer-static`. The `libmpv` package contains `libmpv*.dll`, `libmpv*.dll.a` (a MinGW import library), headers, and the license file (`LICENSE.GPL` or `LICENSE.LGPL`). Variants `gpl` and `lgpl`; targets `win64` and `winarm64` (`.github/workflows/build.yml`, job `mingw64-full`). Features enabled: amf, d3d11, javascript, lcms2, libbluray, libcurl, shaderc, sixel, spirv-cross, subrandr, vapoursynth, vulkan, win32-smtc, zimg; the GPL variant also dvda, dvdnav, rubberband. `-Dgpl=$gpl` is set from the variant name.
  Sources: https://raw.githubusercontent.com/mpv-player/mpv/master/ci/build-mingw64-full.sh, https://raw.githubusercontent.com/mpv-player/mpv/master/.github/workflows/build.yml
- **Publishing status** (V). `ci/publish-release.sh` publishes these as an "Automated development build of the latest master" with a warning that they are "untested development builds ... not official releases". Its notes say: "`libmpv-2.dll` with import library and headers. GPL builds are GPLv3, mpv with all its GPL parts and FFmpeg with `--enable-gpl --enable-version3`. LGPL builds are LGPLv3, made with `-Dgpl=false` against the LGPL FFmpeg variant, for software that cannot link GPL."
  Source: https://raw.githubusercontent.com/mpv-player/mpv/master/ci/publish-release.sh
- **FFmpeg-Builds variants** (V): `gpl` (all dependencies), `lgpl` (no GPL-only libraries such as x264 and x265), `nonfree`, plus `-shared` forms. Source: https://raw.githubusercontent.com/BtbN/FFmpeg-Builds/master/README.md
- **zhongfly/mpv-winbuild** (V). Hourly GitHub Actions builds. `mpv-dev-xxxx.7z` is libmpv with `libmpv-2.dll`; `mpv-dev-lgpl-xxxx.7z` is an LGPLv2.1+ libmpv that "statically links to ffmpeg under LGPLv3". The author states it is not legal advice. Release retention is the last 30 days only. The package list is shared with shinchiro and includes ANGLE. Source: https://raw.githubusercontent.com/zhongfly/mpv-winbuild/main/README.md
- **shinchiro/mpv-winbuild-cmake** (V). CMake superbuild with a long package list (FFmpeg, x264, x265, rubberband, ANGLE, and so on). mpv's own build docs point to it. Sources: https://raw.githubusercontent.com/shinchiro/mpv-winbuild-cmake/master/README.md, https://raw.githubusercontent.com/mpv-player/mpv/master/DOCS/compile-windows.md
- **libmpv2 crate** (V). The `libmpv2-sys` build script emits `cargo:rustc-link-lib=mpv`, so an import library named `mpv` must be findable. The optional `build_libmpv` feature with `MPV_SOURCE` on Windows only adds a link search path for `MPV_SOURCE/64` or `/32`. Crate license is LGPL-2.1. Sources: https://raw.githubusercontent.com/kohsine/libmpv2-rs/master/libmpv-sys/build.rs, https://raw.githubusercontent.com/kohsine/libmpv2-rs/master/build.rs, https://raw.githubusercontent.com/kohsine/libmpv2-rs/master/Cargo.toml
- **Qt deployment** (V). `windeployqt` deploys Qt libraries, `opengl32sw.dll` (software fallback; skip with `--no-opengl-sw`) and the D3D compiler DLLs. It does not deploy ANGLE (`libEGL.dll`, `libGLESv2.dll`) and does not sign (the only signing option is for appx). Source: https://raw.githubusercontent.com/qt/qtbase/dev/src/tools/windeployqt/main.cpp
- **(I)** Runtime layout: put `libmpv-2.dll` next to the exe and run `windeployqt` for Qt.
- **(I) open point**: the upstream import library is a MinGW `.dll.a`. Whether a Rust `x86_64-pc-windows-msvc` target can link it needs a test. Options: use the `x86_64-pc-windows-gnu` target, generate an MSVC `mpv.lib` from the DLL exports, or test lld. Not tested here.

### 2.2 macOS

- **mpv upstream CI** (V). Job `macos` builds on Homebrew dependencies (`ffmpeg-full`, `libass`, `libplacebo`, `molten-vk`, and others) on separate runners `macos-15` (arm), `macos-15-intel`, `macos-26`, `macos-27` and uploads one `mpv.tar.gz` (containing `mpv.app`) per runner, named `macos-15-arm`, `macos-15-intel`, and so on. No universal build and no standalone libmpv package for macOS is published by that workflow. `ci/build-macos.sh` has no `lipo` step. Sources: https://raw.githubusercontent.com/mpv-player/mpv/master/.github/workflows/build.yml, https://raw.githubusercontent.com/mpv-player/mpv/master/ci/build-macos.sh
- **mpv's bundle tooling** (V). `TOOLS/osxbundle.py` calls `TOOLS/dylib_unhell.py`, which walks `otool -L`, copies non-system dylibs to `Contents/MacOS/lib`, and rewrites install names to `@executable_path/lib/...` with `install_name_tool`. Then it signs every file and the bundle with an ad-hoc identity (`codesign --force -s -`). So the upstream bundle is ad-hoc signed, not Developer ID signed or notarized. Sources: https://raw.githubusercontent.com/mpv-player/mpv/master/TOOLS/osxbundle.py, https://raw.githubusercontent.com/mpv-player/mpv/master/TOOLS/dylib_unhell.py
- **media-kit/libmpv-darwin-build** (V). Prebuilt libmpv for macOS and iOS, "compatible with commercial use for playback, and GPL use for encoding". File names include `macos-universal-video-default`, `-full`, `-encodersgpl`, and `macos-arm64` / `macos-amd64`. Minimum macOS 10.9 (set by `uchardet`). Built with Nix. How the universal set is made (lipo or per-arch merge) I did not find in the files I could read. Source: https://raw.githubusercontent.com/media-kit/libmpv-darwin-build/main/README.md
- **IINA** (V). Its build downloads prebuilt dylibs, "by default ... universal binaries", with `--arch universal|arm64|x86_64`, from iina.io. The Xcode steps copy the dylibs into the app in a "Copy Dylibs" build phase. Source: https://raw.githubusercontent.com/iina/iina/develop/README.md
- **Homebrew** (V). The `mpv` formula depends on `ffmpeg`, `libass`, `libplacebo`, and others; license `all_of GPL-2.0-or-later, LGPL-2.1-or-later`. Its bottle list at HEAD has only arm64 entries for macOS. The `ffmpeg` formula is `GPL-3.0-or-later` and configures `--enable-gpl --enable-version3`. Sources: https://raw.githubusercontent.com/Homebrew/homebrew-core/HEAD/Formula/m/mpv.rb, https://raw.githubusercontent.com/Homebrew/homebrew-core/HEAD/Formula/f/ffmpeg.rb
- **(I) universal plan**: build libmpv and its dependency set once per architecture (arm64 on an arm runner, x86_64 on an Intel runner or under Rosetta), then `lipo -create` each dylib pair into one universal dylib, and build the Rust binary for both targets and `lipo` it. Run the `dylib_unhell`-style rewrite once on the merged tree. The alternative is two per-arch DMGs. Cost: two dependency builds. Not tested here.
- **OpenGL on macOS** (V). Apple: "The APIs in the OpenGL and OpenCL frameworks are deprecated and remain present for compatibility purposes" (macOS 10.14 release notes). mpv `render_gl.h` still lists "macOS: CGL is required" for hardware decoding, and mpv builds a `videotoolbox-gl` interop (`hwdec_mac_gl.c`) when that feature is on. Sources: https://developer.apple.com/tutorials/data/documentation/macos-release-notes/macos-mojave-10_14-release-notes.json, https://raw.githubusercontent.com/mpv-player/mpv/master/include/mpv/render_gl.h

### 2.3 Linux

- **Flatpak, mpv itself** (V). `flathub/io.mpv.Mpv` (runtime `org.freedesktop.Platform` 25.08) builds its own libass 0.17.5, libplacebo, FFmpeg `n9.0.1` with `--enable-gpl --enable-version3` and many external libraries (x264, x265, libsmbclient, and others), and mpv `v0.41.0` with `-Dlibmpv=true -Dcplayer=true` and no `-Dgpl=false` (so GPL). Use it as the template for our FFmpeg and libmpv modules. Source: https://raw.githubusercontent.com/flathub/io.mpv.Mpv/master/io.mpv.Mpv.yml
- **Runtimes do not carry libmpv** (V). I read the KDE runtime recipe at every branch from `qt6.5` to `qt6.12` and found no `mpv`, `ffmpeg`, or `libass` text. In `freedesktop-sdk` master, `elements/components/mpv.bst`, `libmpv.bst`, and `libass.bst` do not exist (`ffmpeg.bst` does exist on master; I did not check the 25.08 branch). So our Flatpak must build libmpv and its dependencies as modules (I).
  Sources: https://raw.githubusercontent.com/KDE/flatpak-kde-runtime/qt6.10/org.kde.Sdk.json.in (and other branches), https://gitlab.com/freedesktop-sdk/freedesktop-sdk/-/raw/master/elements/components/ffmpeg.bst
- **AppImage** (V for the exclude list only). The community exclude list names libraries that must come from the host: `libGL.so.1`, `libEGL.so.1`, `libGLX`, `libdrm.so.2`, `libgbm.so.1`, `libwayland-client.so.0`, `libX11`, `libxcb`, `libasound.so.2`, `libfontconfig.so.1`, `libfreetype.so.6`, `libharfbuzz.so.0`, `libz.so.1`, `libstdc++.so.6`, `libc.so.6`. linuxdeploy-plugin-qt documents Wayland plugins as an opt-in. Sources: https://raw.githubusercontent.com/AppImageCommunity/pkg2appimage/master/excludelist, https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-qt/master/README.md
  **(I)**: libmpv must be built against library versions compatible with the host's, on an old glibc baseline. Hardware decode in the AppImage depends on host libva, libvdpau, and Mesa. Not tested here.

### 2.4 Which libmpv build settings matter for us

- `-Dgpl` defaults to `true`; `-Dlibmpv` defaults to `true`; `-Dcplayer` defaults to `true` and can be turned off for a library-only build (V, `meson.options`).
- What `-Dgpl=false` removes (V, mpv `Copyright`): Linux X11 video output, OSS audio, vdpau, jack, DVD, CDDA, DVB, CACA, and the legacy direct3d VO. The same file says `-Dgpl=false` "does not in itself create a LGPLv2.1+ license grant" and that linked libraries (for example a GPL FFmpeg) can change the final license.
- **(I) consequence for Linux**: an `-Dgpl=false` libmpv loses X11 video output; #8 keeps X11-only features, so use the GPL build on Linux. That also fits the project license.
- The client API headers (`client.h`) are ISC licensed so wrappers can use them, while the core is GPLv2+ unless built `-Dgpl=false` (V, header comment).

---

## 3. Hardware decode on Windows (Qt 6, OpenGL render API)

- **Qt 6 has no ANGLE** (V). In qtbase `dev`, `src/plugins/platforms/windows/qwindowsopengltester.cpp` prints "Qt::AA_UseOpenGLES is no longer supported in Qt 6" and "QT_OPENGL=angle is no longer supported in Qt 6". The valid values are `desktop` and `software`. The file `qwindowseglcontext.cpp` exists in qtbase 5.15 and returns 404 in 6.5 and `dev`. `windeployqt` deploys no ANGLE DLLs. Source: https://raw.githubusercontent.com/qt/qtbase/dev/src/plugins/platforms/windows/qwindowsopengltester.cpp
- **mpv's zero-copy D3D11 path needs ANGLE** (V). `hwdec_d3d11egl.c` (built only when `egl-angle` and `d3d-hwaccel` are present) calls `eglGetCurrentDisplay()` and requires the ANGLE extensions `EGL_ANGLE_d3d_share_handle_client_buffer` and `EGL_ANGLE_stream_producer_d3d_texture`; `init` fails otherwise. The mpv manual lists `d3d11va` as requiring `--gpu-context=d3d11` or `--gpu-context=angle`. `render_gl.h` says "Windows: ANGLE is required (although in theory GL/DX interop could be used)". Sources: https://raw.githubusercontent.com/mpv-player/mpv/master/video/out/opengl/hwdec_d3d11egl.c, https://raw.githubusercontent.com/mpv-player/mpv/master/DOCS/man/options.rst, https://raw.githubusercontent.com/mpv-player/mpv/master/include/mpv/render_gl.h
- **The WGL/DX interop path is tied to mpv's own context** (V). `hwdec_dxva2gldx.c` (DXVA2 only, not d3d11va) requires `MPGL_CAP_DXINTEROP` and the native resource `dxinterop_device_HANDLE`, which mpv's own `context_dxinterop.c` provides. A Qt-created desktop GL context passed through the render API does not provide them (I).
- **Result (I)**: with Qt 6 desktop OpenGL plus the libmpv OpenGL render API, use `hwdec=d3d11va-copy` (decode on GPU, copy to system memory, upload) or software decode. Zero-copy `d3d11va` is not available on this path. This sharpens the "ANGLE or WGL interop" cost note in #8: ANGLE is not an option with Qt 6, and WGL interop is not wired to the render API.
- **Other option (V)**: libmpv now has `MPV_RENDER_API_TYPE_D3D11` (`render.h`; `libmpv_d3d11.c` is built when the `d3d11` feature is on, and the official Windows libmpv enables `d3d11`). A zero-copy Windows path would give libmpv a D3D11 device, for example from Qt's RHI. This is the "D3D11 deferred" item in #8. Whether Qt 6 Widgets can host a D3D11 swap chain inside a `QWidget` for this was not researched. Source: https://raw.githubusercontent.com/mpv-player/mpv/master/include/mpv/render.h

---

## 4. Licensing: what a bundled libmpv requires of a GPL-2.0-or-later app

### 4.1 Component licenses (from each project's own license file)

| Component | License (V) | Source file read |
|---|---|---|
| libmpv core (default) | GPLv2+; LGPLv2.1+ only if built `-Dgpl=false` and no GPL FFmpeg | mpv `Copyright` |
| mpv client API headers | ISC | `include/mpv/client.h` |
| `libmpv2`, `libmpv2-sys` crates | LGPL-2.1 | `Cargo.toml` files |
| FFmpeg | LGPL-2.1+; GPL-2+ with `--enable-gpl`; GPL-3 with `--enable-gpl --enable-version3` | FFmpeg `LICENSE.md` |
| x264, x265, libcdio, libxvid and others | GPL v2 per FFmpeg's list; need `--enable-gpl` | FFmpeg `LICENSE.md` |
| librubberband (in FFmpeg's list) and Rubber Band itself | GPL v2 or later, or a commercial licence | FFmpeg `LICENSE.md`, rubberband `README.md` |
| libsmbclient | GPL v3; needs `--enable-gpl --enable-version3` | FFmpeg `LICENSE.md` |
| libass | ISC | libass `COPYING` |
| libplacebo | LGPL-2.1 or later | libplacebo `LICENSE`, `README.md` |
| FriBidi | LGPL-2.1 | fribidi `COPYING` |
| HarfBuzz | "Old MIT" | harfbuzz `COPYING` |
| FreeType | FTL or GPLv2+ (choose one); FTL is incompatible with GPLv2, compatible with GPLv3 | freetype `LICENSE.TXT` |
| Little CMS 2 | MIT | Little-CMS `LICENSE` |
| LuaJIT | MIT | LuaJIT `COPYRIGHT` |
| dav1d | BSD-2-Clause | dav1d `COPYING` |
| libarchive | BSD-style, per-file notices | libarchive `COPYING` |
| zimg | WTFPL | zimg `COPYING` |
| shaderc, SPIRV-Cross, Vulkan-Loader | Apache-2.0 | their `LICENSE` files |

URL pattern: `https://raw.githubusercontent.com/<org>/<repo>/<branch>/<file>`, with `libass/libass`, `haasn/libplacebo`, `fribidi/fribidi`, `harfbuzz/harfbuzz`, `freetype/freetype`, `mm2/Little-CMS`, `LuaJIT/LuaJIT` (branch `v2.1`), `videolan/dav1d`, `libarchive/libarchive`, `sekrit-twc/zimg`, `google/shaderc`, `KhronosGroup/SPIRV-Cross`, `KhronosGroup/Vulkan-Loader` (`LICENSE.txt`), `breakfastquay/rubberband` (branch `default`), `FFmpeg/FFmpeg` (`LICENSE.md`).

Apache compatibility (V, apache.org): "Apache 2 software can therefore be included in GPLv3 projects"; the FSF "has never considered the Apache License to be compatible with GPL version 2". FFmpeg's `LICENSE.md` agrees: Apache 2.0 libraries need `--enable-version3`. Source: https://www.apache.org/licenses/GPL-compatibility.html

### 4.2 What this means for the project (I, built on the V facts above)

1. **A fully featured libmpv build is a GPLv3 combination.** mpv's own Windows GPL libmpv says so (V). Shaderc, SPIRV-Cross, and the Vulkan loader are Apache-2.0 and combine only with GPLv3 or LGPLv3 (V). Our license is GPL-2.0-or-later, so recipients may take it under GPLv3 and the combination is allowed. The project should state in its docs that binary releases are distributed under GPLv3 terms, and not call those binaries "GPL-2.0-only".
2. **Do not add GPL-2.0-only code to the same binary.** FFmpeg's list says "GPL version 2" for x264, x265, and others; I could not confirm which are "v2 only" and which are "v2 or later". Stay with the dependency set that mpv and FFmpeg ship in their own GPLv3 builds, since those projects already made that call.
3. **An LGPL libmpv (`-Dgpl=false` plus LGPL FFmpeg) does not avoid the question.** mpv's own LGPL libmpv is "LGPLv3" (V, publish script). A GPL-2.0-or-later app can use it. It does not keep GPLv2 compatibility either, because the official recipes include Apache-2.0 parts.
4. **FreeType**: pick the GPL option (V: FTL is incompatible with GPLv2). In a GPLv3 combination the FTL is also usable (V), so this is a documentation point, not a blocker.
5. **Source offer.** GPLv2 section 3 and GPLv3 section 6 require that anyone who gets binaries can get the Corresponding Source. Accepted ways include accompanying source, or a written offer valid for at least three years (V, `COPYING.GPLv3` section 6 and mpv `LICENSE.GPL` section 3). For a GitHub-hosted project, the practical form is: tag the exact app commit, and publish the exact source (or a pinned recipe plus source archives) of libmpv, FFmpeg, and every other bundled library for each release, and keep them for at least three years (I). Recipes that point to "latest master" do not meet this (I).
6. **Notices.** Ship `LICENSE.GPL` plus a third-party notices file listing each bundled library, its license text, and its copyright line. ISC, MIT, and BSD licenses require the notice to be kept in copies and binary distributions (V from the texts read: ISC says the copyright and permission notices must "appear in all copies"; dav1d and libarchive are BSD-style with redistribution conditions). FFmpeg's `LICENSE.md` also says that if only executables are distributed, the IJG must be credited in the documentation for three libjpeg-derived files (V).
7. **GPLv3 "Installation Information"** applies to User Products that lock down modified software (V, `COPYING.GPLv3`). A desktop app is not such a product (I). No action.
8. **Never enable FFmpeg `--enable-nonfree`** (for example fdk-aac): the binary becomes "unredistributable" (V, FFmpeg `LICENSE.md`). The FFmpeg-Builds `nonfree` variants must not be used (V that they exist).
9. **Patent-encumbered codecs** are not a copyright-license matter. The sources I read give no guidance. Out of scope; flag for the project owner (I).

---

## 5. Code signing

### 5.1 Windows

- **SignPath Foundation, Open Source Code Signing** (V). The SignPath docs list an edition "Open Source Code Signing" with "1 standard (OV) code signing certificate issued to SignPath Foundation", unlimited projects ("OSS teams must apply for each project individually"), 1 CI pipeline per project, fair-use signing requests, Authenticode supported, and no driver signing because EV certificates are not available to OSS projects. The docs also say the Foundation "provides certificates to individual OSS projects", with origin verification tying a signing request to the repository and a trusted build system; GitHub is a supported trusted build system. Sources: `docs/_data/editions.yml`, `docs/origin-verification/index.md`, `docs/trusted-build-systems/index.md` in https://github.com/signpath/documentation (HEAD of 2026-10-12). The signed binary shows the SignPath Foundation as publisher (I, from "issued to SignPath Foundation"). **Not verified**: eligibility rules (license list, activity, "no commercial"), approval time, and any cost (signpath.org is blocked).
- **Microsoft Artifact Signing** (V; formerly Trusted Signing). Facts from the Microsoft docs source (`MicrosoftDocs/azure-docs`, `articles/artifact-signing/`): needs a paid Azure subscription (free, trial, and sponsored subscriptions are not supported); Public Trust certificates for organizations in the US, Canada, EU, UK, Australia, New Zealand, Japan, South Korea, Singapore, Switzerland, Norway, and Israel; individual developers "must be located in the United States or Canada"; identity validation cannot be expedited; Basic and Premium SKUs; signing from SignTool (Windows SDK 10.0.2261.755 or later, plus a dlib and the .NET 8 runtime), PowerShell, GitHub Actions, or Azure DevOps; the SignTool example uses `/fd SHA256 /tr http://timestamp.acs.microsoft.com /td SHA256`; the certificate CN is the validated legal name, with no custom CN or O. **Not verified**: price (azure.microsoft.com is blocked).
- **Traditional OV/EV certificate from a CA**: not researched (no reachable primary source). Cost and hardware token rules are unknown (section 9).
- **Timestamping** (V for the Microsoft example, I for the rule): always timestamp with RFC 3161 (`/tr` and `/td SHA256`) so the signature stays valid after the certificate expires. I did not read the certificate validity period for Artifact Signing.
- **SmartScreen** (V). Microsoft's FAQ says SmartScreen reputation "builds up automatically", and prompts stop after the file hash has enough download history. A signature alone does not remove first-run prompts for a new publisher (I from V).
- **What to sign (I)**: the app exe, `libmpv-2.dll`, any other DLL we build or repackage, and the installer. I did not check whether the Qt DLLs from the Qt installer are already signed.

### 5.2 macOS

- **Cost** (V). Apple Developer Program: "99 USD per membership year"; a fee waiver can be requested by "a nonprofit organization, accredited educational institution, or government entity" that meets requirements. There is no open source exemption in that text. Developer ID certificates come with the paid membership. Sources: https://developer.apple.com/programs/enroll/, https://developer.apple.com/developer-id/
- **Notarization requirements** (V, Apple "Notarizing macOS software before distribution"): sign all executables with a Developer ID certificate (not ad hoc, not development); enable Hardened Runtime; include a secure timestamp; do not set `com.apple.security.get-task-allow` to true; link against the macOS 10.9 or later SDK; use properly formatted entitlements. Apps, disk images (UDIF), flat installer packages, and ZIP archives can be notarized. Source: https://developer.apple.com/tutorials/data/documentation/security/notarizing-macos-software-before-distribution.json
- **Steps** (V, Apple "Customizing the notarization workflow" and "Resolving common notarization issues"): `ditto -c -k --keepParent App.app App.zip`; `xcrun notarytool submit App.zip --keychain-profile <profile> --wait`; `xcrun notarytool log <id>` to read errors; `xcrun stapler staple` the ticket; check with `codesign -vvv --deep --strict`. The secure timestamp comes only from `timestamp.apple.com`, so the signing machine needs internet access. `altool` is not accepted since 2023-11-01; use `notarytool`. Credentials: Apple ID with an app-specific password, stored with `notarytool store-credentials`. Sources: https://developer.apple.com/tutorials/data/documentation/security/customizing-the-notarization-workflow.json, https://developer.apple.com/tutorials/data/documentation/security/resolving-common-notarization-issues.json
- **Qt tooling** (V). `macdeployqt` (qtbase `dev`, `src/tools/macdeployqt`) has `-codesign=<ident>`, `-hardened-runtime`, `-timestamp`, `-sign-for-notarization=<ident>` (which sets hardened runtime and timestamp), `-libpath=`, and `-executable=`. It signs with `codesign --preserve-metadata=identifier,entitlements --force -s <id> [-o runtime] [--timestamp]`. It resolves and deploys Qt frameworks. Whether it also copies non-Qt dylibs such as libmpv is not clear from the code I read; treat it as Qt-only, bundle libmpv separately, then sign everything in one final pass (I). Source: https://raw.githubusercontent.com/qt/qtbase/dev/src/tools/macdeployqt/macdeployqt/main.cpp and `.../shared/shared.cpp`
- **Entitlements for libmpv and Qt plugins** (V for the rules, I for what the app needs):
  - Apple, Hardened Runtime: "Shared libraries, frameworks, and in-process plug-ins inherit the entitlements of their host executable"; entitlements go only on executables. Library validation is on by default and only lets the process load libraries "signed by Apple or signed with the same Team ID as the main executable". So if every bundled dylib (libmpv, FFmpeg, libass, Qt frameworks, Qt plugins) is re-signed with our own Developer ID, `disable-library-validation` is not needed (I from V). Do not add it "unnecessarily", because Gatekeeper then runs extra checks (V).
  - `com.apple.security.cs.allow-jit` is needed only for code that uses `MAP_JIT` (V). libmpv can be built with LuaJIT (`-Dlua=luajit` appears in mpv's Windows CI). Whether LuaJIT on Apple Silicon needs `allow-jit` under Hardened Runtime I did not verify. Test with Hardened Runtime on; if Lua scripting fails, add `allow-jit` or build libmpv against plain Lua or without Lua. Our app (Widgets, no QML JIT) does not otherwise need JIT (I).
  - `allow-unsigned-executable-memory` and `allow-dyld-environment-variables` should not be needed (I). Apple says to add only entitlements that are "absolutely necessary".
- **Ad hoc signing is not enough** for notarization (V). Every Mach-O in the bundle must be signed with the Developer ID, nested dylibs and frameworks first and the app last (I).

---

## 6. Flatpak: KDE runtime branches and Qt 6 versions

From `KDE/flatpak-kde-runtime` (GitHub mirror of `invent.kde.org/packaging/flatpak-kde-runtime`). Its README says "There's a different branch for each version of Qt." For each branch I read `org.kde.Sdk.json.in` at the branch tip (2026-10-15): `branch`, `runtime-version` (the freedesktop base), and the qtbase source tarball.

| KDE runtime | Qt 6 in recipe | freedesktop base |
|---|---|---|
| 6.5 | 6.5.3 | 22.08 |
| 6.6 | 6.6.3 | 23.08 |
| 6.7 | 6.7.3 | 23.08 |
| 6.8 | 6.8.3 | 24.08 |
| 6.9 | 6.9.3 | 24.08 |
| 6.10 | 6.10.3 | 25.08 |
| 6.11 | 6.11.2 | 25.08 |
| 6.12-beta | 6.12.0 | 26.08 |

(V for the table. The #8 prototype ran on Qt 6.11, which maps to runtime 6.11.)

- The runtime provides Qt (qtbase, qtwayland, qtmultimedia, qtsvg and others) but not libmpv (V, section 2.3). `inherit-extensions` in the 6.10 recipe includes `org.freedesktop.Platform.GL`, `VAAPI.Intel`, `VAAPI.nvidia`, `codecs-extra`, and `GStreamer` (V). We would add FFmpeg, libass, libplacebo, and libmpv as app modules (I).
- **Recommendation (I)**: target the newest stable branch (6.11 today) for Flathub and use Qt 6.11 on Windows and macOS so all three OSes match what the prototype tested. Flathub's end-of-life policy for older KDE runtime branches is not verified.
- Permissions for mpv-like use, from the `io.mpv.Mpv` manifest (V): `--socket=wayland`, `--socket=x11`, `--device=all` (GPU and hardware decode), `--socket=pulseaudio`, `--share=network`, `--filesystem=xdg-videos`. Each one needs a justification at Flathub review (I).
- Flathub builds from source inside the sandbox, so the source offer is met by a manifest that pins exact tags and commits (I; the Flathub policy pages are blocked).

---

## 7. Recommendations (all I)

1. Build libmpv ourselves for each OS from pinned mpv and FFmpeg tags: mpv's `ci/build-mingw64-full.sh` pattern on Windows, `build-macos.sh` plus `dylib_unhell.py` ideas on macOS, the `io.mpv.Mpv` module list on Flatpak. Do not depend on a "latest" download (30-day retention at zhongfly, development builds only at mpv).
2. Use the GPL libmpv build everywhere and state that binaries are under GPLv3 terms (section 4.2). Record the exact source revisions per release for the source offer.
3. Windows: ship `hwdec=d3d11va-copy` by default; treat zero-copy as a later task through the D3D11 render API. Do not plan on ANGLE.
4. Signing: apply to SignPath Foundation for Windows (free for OSS, needs approval). Fall back to Artifact Signing if the owner is an individual in the US or Canada, or an organization in a listed country. macOS needs the 99 USD per year membership; I found no free route.
5. macOS: decide early between one universal bundle (`lipo` pipeline) and two per-arch downloads; the universal route doubles the dependency build.
6. Before the first release, run a notarized build with Hardened Runtime and libmpv to settle the LuaJIT question.

---

## 8. Sources read

All fetched 2026-10-15.

- mpv (base `https://raw.githubusercontent.com/mpv-player/mpv/master/`): `meson.build`, `meson.options`, `Copyright`, `LICENSE.GPL`, `include/mpv/{client,render,render_gl}.h`, `DOCS/man/options.rst`, `DOCS/compile-windows.md`, `ci/{build-mingw64-full,build-macos,build-common,publish-release}.sh`, `.github/workflows/build.yml`, `TOOLS/{osxbundle,dylib_unhell}.py`, `video/out/opengl/{hwdec_d3d11egl,hwdec_dxva2gldx,context_dxinterop}.c`.
- FFmpeg `LICENSE.md` and `COPYING.GPLv3` (`FFmpeg/FFmpeg`, master).
- BtbN/FFmpeg-Builds, zhongfly/mpv-winbuild, shinchiro/mpv-winbuild-cmake, media-kit/libmpv-darwin-build, iina/iina READMEs; Homebrew `Formula/m/mpv.rb` and `Formula/f/ffmpeg.rb`; flathub/io.mpv.Mpv `io.mpv.Mpv.yml`; KDE/flatpak-kde-runtime branches `qt6.5` to `qt6.12`; freedesktop-sdk (gitlab.com) `elements/components/ffmpeg.bst`; AppImageCommunity/pkg2appimage `excludelist`; linuxdeploy-plugin-qt `README.md`; kohsine/libmpv2-rs `Cargo.toml`, `build.rs`, `libmpv-sys/{Cargo.toml,build.rs}`.
- License files for libass, libplacebo, fribidi, harfbuzz, freetype, Little-CMS, LuaJIT, dav1d, libarchive, zimg, rubberband, shaderc, SPIRV-Cross, Vulkan-Loader, and the x264 `COPYING` (GPLv2 text only); https://www.apache.org/licenses/GPL-compatibility.html.
- Qt: qtbase `dev` `src/plugins/platforms/windows/{qwindowsopengltester,qwindowsintegration,qwindowsglcontext}.cpp`, `src/tools/windeployqt/main.cpp`, `src/tools/macdeployqt/**`, `dist/changes-6.0.0`; existence checks for `qwindowseglcontext.cpp` on qtbase 5.15, 6.5, and `dev`.
- Apple (developer.apple.com): notarization overview, customizing the notarization workflow, resolving common notarization issues, Hardened Runtime, the `disable-library-validation`, `allow-jit`, and `allow-unsigned-executable-memory` entitlement pages, macOS 10.14 release notes, programs and enrollment pages, Developer ID page.
- Microsoft: `MicrosoftDocs/azure-docs` `articles/artifact-signing/{overview,quickstart,faq,concept-resources-roles,how-to-signing-integrations}`.
- SignPath: https://github.com/signpath/documentation (`docs/_data/editions.yml`, `docs/origin-verification/index.md`, `docs/trusted-build-systems/index.md`).

---

## 9. Not verified

- **Windows signing eligibility and cost**: SignPath Foundation entry rules and approval process; Artifact Signing price and certificate validity period; any traditional CA OV/EV certificate price, hardware token rules, and whether EV still matters for SmartScreen. The project owner's country is unknown, and it decides Artifact Signing eligibility. (signpath.org, azure.microsoft.com, and learn.microsoft.com are blocked.)
- **Whether Qt DLLs from the official installer are already Authenticode signed**, and whether to re-sign them.
- **GPL-version details of FFmpeg-linked libraries**: whether x264, x265, and libsmbclient are "v2 only" or "v2 or later" at the versions we would use. FFmpeg's list says "GPL version 2" without "or later".
- **libbluray, uchardet, and mujs licenses** (license files not found at the paths tried). **Whether the official Windows libmpv DLL includes ANGLE** (mpv's CI does not set `egl-angle` explicitly, and I did not inspect the BtbN image).
- **FSF GPL FAQ and gnu.org texts** (blocked). The GPL reasoning in section 4.2 relies on the GPL texts in the mpv and FFmpeg repos, the FFmpeg and mpv statements, and the Apache page. It is not legal advice.
- **How media-kit and IINA build universal macOS dylibs** (no `lipo` step found in readable files); **whether `macdeployqt` bundles non-Qt dylibs**; **LuaJIT and `allow-jit` behavior** under Hardened Runtime on Apple Silicon; **Homebrew's Intel macOS support policy** (the bottle list shows only arm64 for macOS today).
- **MSVC versus MinGW import library**: whether Rust's MSVC target can link mpv's `libmpv.dll.a`. Needs a test on Windows.
- **Qt 6 Widgets hosting a D3D11 swap chain for libmpv's D3D11 render API**: not researched.
- **AppImage details**: docs.appimage.org is blocked, so glibc baseline guidance, FUSE requirements, and AppImage signing options are unverified; only the exclude list and the linuxdeploy-plugin-qt README were read.
- **Flathub policies** (docs.flathub.org blocked): EOL handling of KDE runtime branches, source-build rules, and permission review expectations.
- **Anything run on real systems**: no Windows or macOS system was available; no build, signature, or notarization was run. This note comes from reading sources only.
