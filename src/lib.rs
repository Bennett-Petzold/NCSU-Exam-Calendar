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
