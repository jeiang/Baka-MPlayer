// PROTOTYPE: stands in for src/ui/mainwindow.cpp. Only builds the window from the uic output.
#include <QApplication>
#include <QMainWindow>
#include <QPixmap>
#include <QTimer>
#include <cstdlib>
#include <QVBoxLayout>
#include "mpvglwidget.h"
#include "ui_mainwindow.h"

namespace {
class ProtoWindow : public QMainWindow {
public:
    ProtoWindow() {
        ui.setupUi(this);
        // The original embeds mpv by window id into mpvFrame; here a QOpenGLWidget fills it instead.
        auto *l = new QVBoxLayout(ui.mpvFrame);
        l->setContentsMargins(0, 0, 0, 0);
        l->addWidget(new MpvGLWidget(ui.mpvFrame));
    }
    Ui::MainWindow ui;
};
}

extern "C" int proto_run(int argc, char **argv) {
    QApplication app(argc, argv);
    ProtoWindow w;
    w.resize(1000, 640);
    w.show();
    if (const char *shot = std::getenv("PROTO_SHOT")) {
        QTimer::singleShot(3000, [&] { w.grab().save(shot); app.quit(); });
    }
    return app.exec();
}
