use requests;
use scraper::{Html, Selector};
use std::collections::HashSet;
use async_std::task;
use futures::future::{BoxFuture, FutureExt};


pub fn run() {
    println!("Hello Sputnik");
    let sitemap = Sitemap::new("https://www.wassersport-holnis.de");
    println!("{:?}", sitemap.links);
}

struct Sitemap<'a> {
    base_url: &'a str,
    links: HashSet<String>,
}

impl<'a> Sitemap<'a> {
    pub fn new(base_url: &'a str) -> Sitemap {
        let mut sitemap = Sitemap {
            base_url,
            links: HashSet::new()
        };

        let links = Sitemap::discover(base_url, &base_url.to_string());
        println!("{:?}", links);
        sitemap
    }

    fn discover(base_url: &str, link: &String) -> Vec<String> {
        let response = requests::get(Sitemap::build_url(base_url, link)).unwrap();
        let document = Html::parse_document(response.text().unwrap());
        let selector = Selector::parse("a").unwrap();

        let anchors = document.select(&selector);

        anchors.filter_map(|anchor| {
            match anchor.value().attr("href") {
                Some(href) => {
                    if Sitemap::is_internal_http_link(base_url, href) {
                        Some(href.to_string())
                    } else {
                        None
                    }
                },
                None => None
            }
        }).collect()
    }

    fn is_external_link(base_url: &str, link: &str) -> bool {
        (link.contains("http") || link.contains("https")) && !link.contains(base_url)
    }

    fn is_internal_http_link(base_url: &str, link: &str) -> bool {
        !Sitemap::is_external_link(base_url, link)
        && !link.contains("mailto:")
        && !link.contains("tel:")
        && link != "#"
    }

    fn build_url(base_url: &str, link: &String) -> String {
        if !link.contains("http") {
            format!("{}{}", String::from(base_url), link)
        } else {
            link.to_string()
        }
    }
}
