use cxx_qt_build::{CxxQtBuilder};

fn main() {
    CxxQtBuilder::new()
        // Link Qt's Network library
        // - Qt Core is always linked
        // - Qt Gui is linked by enabling the qt_gui Cargo feature of cxx-qt-lib.
        // - Qt Qml is linked by enabling the qt_qml Cargo feature of cxx-qt-lib.
        // - Qt Qml requires linking Qt Network on macOS
        // .qt_module("Network")
        .files(["src/cxxqt_object.rs"])
        .build();
}