# Research: libmpv render API inside a Qt window from Rust
Type: research
Status: open
Blocked by:

## Question

How do Rust apps use the libmpv render API (OpenGL, and Vulkan or Metal where available) inside a Qt widget on Windows, Linux (X11 and Wayland), and macOS? Compare Rust mpv crates (libmpv2, libmpv-sys, others) for render-API support and maintenance. List what `MpvHandler` uses today (`wid` embedding, OSD overlays in `src/overlayhandler.cpp`) and what has to change. Note hardware decoding and HDR constraints per OS.
