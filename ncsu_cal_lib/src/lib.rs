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

use reqwest::Client;
use select::document::Document;

pub mod calendar;
pub mod gui;

async fn get_page_html<S: AsRef<str>>(url: S) -> Result<String, reqwest::Error> {
    let client = Client::builder().build()?;
    let response = client.get(url.as_ref()).send().await?;
    response.text().await
}

pub async fn get_page_document<S: AsRef<str>>(url: S) -> Result<Document, reqwest::Error> {
    let html = get_page_html(url).await?;
    Ok(Document::from(html.as_str()))
}
