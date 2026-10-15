// PROTOTYPE: QOpenGLWidget that hosts libmpv's render API. All mpv logic lives in Rust (src/video.rs);
// this class only provides the GL context and forwards init/paint.
#pragma once
#include <QOpenGLWidget>

extern "C" {
void proto_mpv_init(void *widget);       // implemented in Rust: create Mpv + render context
void proto_mpv_paint(int fbo, int w, int h); // implemented in Rust
void proto_mpv_shutdown();               // implemented in Rust
}

class MpvGLWidget : public QOpenGLWidget {
    Q_OBJECT
public:
    explicit MpvGLWidget(QWidget *parent = nullptr);
    ~MpvGLWidget() override;
    void *getProcAddress(const char *name);

protected:
    void initializeGL() override;
    void paintGL() override;
};
