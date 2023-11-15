use std::fs::File;

use ncsu_exam_lib::calendar::get_calendars;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cals = get_calendars("https://studentservices.ncsu.edu/calendars/exam-calendar/")
        .await
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&cals).unwrap());
    let create = File::create("exams.json").unwrap();
    serde_json::to_writer(create, &cals).unwrap();
}
