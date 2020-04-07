#![feature(async_closure)]
use scraper::{element_ref::ElementRef, Html, Selector};
use futures::{stream::{self, StreamExt},future::BoxFuture};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let links_in_page = crawl("http://www.parliament.gov.zm").await.await.expect("Could not crawl through the web");
    println!("links in page{:?}", links_in_page);
    Ok(())
}

async fn crawl(website: &str) -> BoxFuture<'static,Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>> {
    let html = match reqwest::get(website).await.map(|t|t.text())
    {
        Ok(txt)=>txt,
        Err(err)=> return Box::pin(err)
    };
    let document = Html::parse_document(&html.await.unwrap_or_default());
    let selector = Selector::parse("link").unwrap();
    let link_stream = stream::iter(document.select(&selector));
    let links = link_stream.fold(
        vec![],
        |link_vector, current_link: ElementRef|async move {
            if let Some(link) = current_link.value().attr("href") {
                if link.is_empty() {
                    link_vector
                } else {
                    let crawled_links = (crawl(link).await).await.unwrap_or_default();
                    link_vector
                        .into_iter()
                        .chain(vec![link.to_string()])
                        .chain(crawled_links)
                        .collect::<Vec<_>>()
                }
            } else {
                link_vector
            }
        },
    ).await;
    Box::pin(Ok(links))
}
