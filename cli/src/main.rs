/*
* Copyright (C) 2023 Bennett Petzold
*
* This file is part of ncsu_exam_calendar.
*
* ncsu_exam_calendar is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 2 of the License, or (at your option) any later version.
*
* ncsu_exam_calendar is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
*
* You should have received a copy of the GNU General Public License along with ncsu_exam_calendar. If not, see <https://www.gnu.org/licenses/>.
*/

use std::fs::File;

use ncsu_cal_lib::calendar::get_calendars;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cals = get_calendars("https://studentservices.ncsu.edu/calendars/exam-calendar/")
        .await
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&cals).unwrap());
    let create = File::create("exams.json").unwrap();
    serde_json::to_writer(create, &cals).unwrap();
}
