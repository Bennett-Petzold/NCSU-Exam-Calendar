use log::LevelFilter;
use ncsu_exam_lib::gui::app;

fn main() {
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    dioxus_desktop::launch(app);
}
