use requests;
use scraper::{Html, Selector};
use std::collections::HashSet;
use async_std::task;
use futures::future::{BoxFuture, FutureExt};
use async_recursion::async_recursion;



pub fn run() {
    println!("Hello Sputnik");
    let sitemap = Sitemap::new("https://www.test.de");
    println!("{:?}", sitemap.links);
    println!("Found {:?} links", sitemap.links.len());
}

struct Sitemap {
    base_url: &'static str,
    links: HashSet<String>,
}

impl Sitemap {
    pub fn new(base_url: &'static str) -> Sitemap {
        let mut sitemap = Sitemap {
            base_url,
            links: HashSet::new()
        };

        sitemap.links = Sitemap::discover(base_url, None);
        task::block_on(sitemap.discover_multiple(base_url, sitemap.links.clone()));
        sitemap
    }

    #[async_recursion]
    async fn discover_multiple(&mut self, base_url: &'static str, links: HashSet<String>) {
        let mut handles = Vec::new();
        for link in links {
            if Sitemap::is_internal_http_link(base_url, &link) {
                handles.push(task::spawn(async move {
                    Sitemap::discover(base_url, Some(&link))
                }));
            }
        }

        let mut new_links = HashSet::new();
        for handle in handles {
            new_links = (&handle.await).difference(&self.links).map(|link| link.to_string()).collect();
        }

        self.links = self.links.union(&new_links).map(|link| link.to_string()).collect::<HashSet<String>>();

        if new_links.len() > 0 {
            self.discover_multiple(base_url, new_links).await;
        }

    }

    fn discover(base_url: &'static str, link_option: Option<&String>) -> HashSet<String> {
        let mut url = base_url.to_string();

        if let Some(link) = link_option {
            url = Sitemap::build_url(base_url, link);
        }

        let response = requests::get(url).unwrap();
        let document = Html::parse_document(response.text().unwrap());
        let selector = Selector::parse("a").unwrap();

        let anchors = document.select(&selector);

        anchors.filter_map(|anchor| {
            if let Some(href) = anchor.value().attr("href") {
                if Sitemap::is_internal_http_link(base_url, href) {
                    return Some(href.to_string());
                }
            }

            return None;
        }).collect::<HashSet<String>>()
    }

    fn is_external_link(base_url: &'static str, link: &str) -> bool {
        (link.contains("http") || link.contains("https")) && !link.contains(base_url)
    }

    fn is_internal_http_link(base_url: &'static str, link: &str) -> bool {
        !Sitemap::is_external_link(base_url, link)
        && !link.contains("mailto:")
        && !link.contains("tel:")
        && !link.contains("#")
    }

    fn build_url(base_url: &'static str, link: &String) -> String {
        if !link.contains("http") {
            format!("{}{}", String::from(base_url), link)
        } else {
            link.to_string()
        }
    }
}
