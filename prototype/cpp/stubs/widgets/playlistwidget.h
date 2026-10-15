// PROTOTYPE stub: the real PlaylistWidget depends on BakaEngine/MpvHandler.
#pragma once
#include <QListWidget>
class PlaylistWidget : public QListWidget {
    Q_OBJECT
public:
    explicit PlaylistWidget(QWidget *parent = nullptr) : QListWidget(parent) {}
};
