#include "mpvglwidget.h"
#include <QOpenGLContext>
#include <QMetaObject>

MpvGLWidget::MpvGLWidget(QWidget *parent) : QOpenGLWidget(parent) {}

MpvGLWidget::~MpvGLWidget() {
    makeCurrent();
    proto_mpv_shutdown(); // render context must die while the GL context is current
    doneCurrent();
}

void *MpvGLWidget::getProcAddress(const char *name) {
    auto *ctx = QOpenGLContext::currentContext();
    return ctx ? reinterpret_cast<void *>(ctx->getProcAddress(name)) : nullptr;
}

void MpvGLWidget::initializeGL() { proto_mpv_init(this); }

void MpvGLWidget::paintGL() {
    const qreal dpr = devicePixelRatioF();
    proto_mpv_paint(int(defaultFramebufferObject()), int(width() * dpr), int(height() * dpr));
}

// Called from Rust (any thread) when mpv has a new frame.
extern "C" void proto_request_update(void *widget) {
    QMetaObject::invokeMethod(static_cast<MpvGLWidget *>(widget), "update", Qt::QueuedConnection);
}
extern "C" void *proto_get_proc_address(void *widget, const char *name) {
    return static_cast<MpvGLWidget *>(widget)->getProcAddress(name);
}
