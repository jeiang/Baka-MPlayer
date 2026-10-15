// PROTOTYPE: the only C++ the Rust SeekBar needs. cxx-qt-lib has no QPainter-on-widget binding.
#pragma once
#include <QSlider>
#include "rust/cxx.h"
void proto_draw_ticks(QSlider *slider, rust::Slice<const int> per_mille);
