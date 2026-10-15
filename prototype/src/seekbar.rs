// PROTOTYPE: SeekBar re-done as a Rust-backed QSlider subclass (cxx-qt). Mirrors src/widgets/seekbar.cpp paintEvent:
// call the base paintEvent, then draw tick lines with QPainter.
#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++Qt" {
        include!(<QtWidgets/QWidget>);
        #[qobject]
        type QWidget;

        include!(<QtWidgets/QSlider>);
        #[qobject]
        #[base = QWidget]
        type QSlider;
    }

    unsafe extern "C++" {
        include!(<QtGui/QPaintEvent>);
        type QPaintEvent;

        include!("rustseekbar_shim.h");
        unsafe fn proto_draw_ticks(slider: *mut QSlider, per_mille: &[i32]);
    }

    extern "RustQt" {
        #[qobject]
        #[base = QSlider]
        type RustSeekBar = super::RustSeekBarRust;
    }

    unsafe extern "RustQt" {
        #[cxx_override]
        #[cxx_name = "paintEvent"]
        unsafe fn paint_event(self: Pin<&mut RustSeekBar>, event: *mut QPaintEvent);

        #[inherit]
        #[cxx_name = "isEnabled"]
        fn is_enabled(self: &RustSeekBar) -> bool;

        #[inherit]
        #[cxx_name = "paintEvent"]
        unsafe fn paint_event_super(self: Pin<&mut RustSeekBar>, event: *mut QPaintEvent);
    }

    impl cxx_qt::Constructor<(*mut QWidget,), BaseArguments = (*mut QWidget,)> for RustSeekBar {}
}

use core::pin::Pin;
use cxx_qt::casting::Upcast;
use ffi::{QPaintEvent, QSlider, QWidget, RustSeekBar};

pub struct RustSeekBarRust {
    ticks: Vec<i32>, // per mille of the slider range
}

impl Default for RustSeekBarRust {
    fn default() -> Self {
        Self { ticks: vec![250, 500, 750] }
    }
}

impl cxx_qt::Constructor<(*mut QWidget,)> for RustSeekBar {
    type BaseArguments = (*mut QWidget,);
    type InitializeArguments = ();
    type NewArguments = ();

    fn route_arguments(args: (*mut QWidget,)) -> (Self::NewArguments, Self::BaseArguments, Self::InitializeArguments) {
        ((), args, ())
    }

    fn new(_: ()) -> RustSeekBarRust {
        RustSeekBarRust::default()
    }
}

impl ffi::RustSeekBar {
    /// Override of QWidget::paintEvent.
    unsafe fn paint_event(mut self: Pin<&mut Self>, event: *mut QPaintEvent) {
        self.as_mut().paint_event_super(event);
        if self.as_ref().is_enabled() {
            let ticks = self.ticks.clone();
            let slider: *mut QSlider = Upcast::<QSlider>::upcast_mut(self.get_unchecked_mut());
            ffi::proto_draw_ticks(slider, &ticks);
        }
    }
}
