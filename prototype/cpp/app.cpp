// PROTOTYPE: stands in for src/ui/mainwindow.cpp. Only builds the window from the uic output.
#include <QApplication>
#include <QMainWindow>
#include <QPixmap>
#include <QSignalBlocker>
#include <QTimer>
#include <clocale>
#include <cstdlib>
#include <QVBoxLayout>
#include "mpvglwidget.h"
#include <QLayout>
#include "ui_mainwindow.h"
#include "proto-main-window/src/seekbar.cxxqt.h"

namespace {
class ProtoWindow : public QMainWindow {
public:
    ProtoWindow() {
        ui.setupUi(this);
        // The original embeds mpv by window id into mpvFrame; here a QOpenGLWidget fills it instead.
        auto *l = new QVBoxLayout(ui.mpvFrame);
        l->setContentsMargins(0, 0, 0, 0);
        l->addWidget(new MpvGLWidget(ui.mpvFrame));

        // PROTO_RUST_SEEKBAR=1 swaps the promoted C++ SeekBar for the Rust-backed one. Both get ticks at 25/50/75%.
        QSlider *seek = ui.seekBar;
        if (std::getenv("PROTO_RUST_SEEKBAR")) {
            auto *old = ui.seekBar;
            auto *nw = new RustSeekBar(old->parentWidget());
            nw->setObjectName(old->objectName());
            nw->setSizePolicy(old->sizePolicy());
            nw->setCursor(old->cursor());
            nw->setFocusPolicy(old->focusPolicy());
            nw->setOrientation(old->orientation());
            nw->setStyleSheet(old->styleSheet());
            nw->setMinimum(old->minimum());
            nw->setMaximum(old->maximum());
            old->parentWidget()->layout()->replaceWidget(old, nw);
            old->deleteLater();
            seek = nw;
        } else {
            ui.seekBar->setTicks({250, 500, 750});
            ui.seekBar->setTracking(1000);
        }
        seek->setEnabled(true);
        seek->setValue(seek->maximum() / 3);

        // The original hides the debug output pane (output box + command line) unless debug mode is on.
        const bool debug = std::getenv("PROTO_DEBUG") != nullptr;
        ui.verticalWidget->setVisible(debug);
        ui.actionShow_D_ebug_Output->setChecked(debug);

        // Stand-in for MpvHandler's `playback-time` observer: follow playback.
        auto *tick = new QTimer(this);
        connect(tick, &QTimer::timeout, this, [seek] {
            const int pm = proto_mpv_progress();
            if (pm >= 0 && !seek->isSliderDown()) {
                QSignalBlocker b(seek);
                seek->setValue(int(double(pm) / 1000 * seek->maximum()));
            }
        });
        tick->start(250);
    }
    Ui::MainWindow ui;
};
}

extern "C" int proto_run(int argc, char **argv) {
    QApplication app(argc, argv);
    // libmpv refuses to start unless LC_NUMERIC is "C"; Qt adopts the system locale on startup.
    std::setlocale(LC_NUMERIC, "C");
    ProtoWindow w;
    w.resize(1000, 640);
    w.show();
    if (const char *shot = std::getenv("PROTO_SHOT")) {
        QTimer::singleShot(std::getenv("PROTO_SHOT_MS") ? std::atoi(std::getenv("PROTO_SHOT_MS")) : 3000, [&] { w.grab().save(shot); app.quit(); });
    }
    return app.exec();
}
