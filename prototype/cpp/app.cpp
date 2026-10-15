// PROTOTYPE: stands in for src/ui/mainwindow.cpp. Only builds the window from the uic output.
#include <QApplication>
#include <QMainWindow>
#include <QPixmap>
#include <QTimer>
#include <cstdlib>
#include "ui_mainwindow.h"

namespace {
class ProtoWindow : public QMainWindow {
public:
    ProtoWindow() { ui.setupUi(this); }
    Ui::MainWindow ui;
};
}

extern "C" int proto_run(int argc, char **argv) {
    QApplication app(argc, argv);
    ProtoWindow w;
    w.resize(w.sizeHint());
    w.show();
    if (const char *shot = std::getenv("PROTO_SHOT")) {
        QTimer::singleShot(500, [&] { w.grab().save(shot); app.quit(); });
    }
    return app.exec();
}
