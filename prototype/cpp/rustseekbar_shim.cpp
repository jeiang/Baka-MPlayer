#include "rustseekbar_shim.h"
#include <QPainter>
#include <QStyle>

void proto_draw_ticks(QSlider *s, rust::Slice<const int> per_mille) {
    QPainter painter(s);
    painter.setPen(QColor(190, 190, 190));
    const QRect r = s->rect();
    for (int pm : per_mille) {
        int x = QStyle::sliderPositionFromValue(s->minimum(), s->maximum(), int(double(pm) / 1000 * s->maximum()), s->width());
        painter.drawLine(x, r.top(), x, r.bottom());
    }
}
