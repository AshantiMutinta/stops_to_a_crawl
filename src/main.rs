#![feature(async_closure)]
#![feature(vec_remove_item)]
use futures::stream::{self, StreamExt};
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Ok(pickle) = PickleDb::load(
        "crawl.db",
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    ) {
        let links_in_page = pickle
            .get_all()
            .into_iter()
            .filter(|s| !pickle.get::<bool>(s).unwrap_or(true))
            .collect::<Vec<String>>();
        crawl_websites(links_in_page, pickle).await
    } else {
        let pickle = PickleDb::new(
            "crawl.db",
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        );
        let links_in_page = crawl("https://moz.com/top500").await;
        crawl_websites(links_in_page, pickle).await
    };

    Ok(())
}

async fn crawl_websites(mut links_in_page: Vec<String>, mut pickle: PickleDb) {
    while !links_in_page.is_empty() {
        let links_iter = links_in_page.clone().into_iter();
        for current_page in links_iter {
            pickle
                .set(&current_page, &false)
                .expect("could not add to pickledb");
            let crawled_pages = crawl(&*current_page).await;
            println!("crawled {:?}", crawled_pages);
            links_in_page.remove_item(&current_page);
            pickle
                .set(&current_page, &true)
                .expect("could not add to pickledb");
            links_in_page.extend(crawled_pages);
        }
    }
}

async fn crawl(website: &str) -> Vec<String> {
    let html = match reqwest::get(website).await.map(|t| t.text()) {
        Ok(txt) => txt,
        Err(_) => return Vec::new(),
    };
    let html = html.await.unwrap_or_default();
    let document = Html::parse_document(&html);
    let selector = Selector::parse("link").unwrap();
    let link_tags = document
        .select(&selector)
        .filter_map(|current_link| {
            current_link
                .value()
                .attr("href")
                .map(|some| some.to_string())
        })
        .collect::<Vec<String>>();

    let link_stream = stream::iter(link_tags);
    let links = link_stream.fold(
        vec![],
        |link_vector: Vec<String>, current_link: String| async move {
            if current_link.is_empty() {
                link_vector
            } else {
                link_vector
                    .into_iter()
                    .chain(vec![current_link.to_string()])
                    .collect::<Vec<_>>()
            }
        },
    );
    links.await
}
