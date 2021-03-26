use requests;
use scraper::{Html, Selector};
use std::collections::HashSet;
use async_std::task;
use std::future::Future;

pub fn run() {
    println!("Hello Sputnik");
    let sitemap = Sitemap::new("https://www.test.de");
    println!("{:?}", sitemap.links);
}

struct Sitemap<'a> {
    base_url: &'a str,
    links: HashSet<String>,
    queue: Vec<String>,
}

impl<'a> Sitemap<'a> {
    pub fn new(base_url: &'a str) -> Sitemap {
        let mut sitemap = Sitemap {
            base_url,
            links: HashSet::new(),
            queue: Vec::new(),
        };

        sitemap.queue.push(sitemap.base_url.to_string());
        task::block_on(sitemap.discover_queued_links());
        sitemap
    }

    async fn discover_queued_links(&mut self) {
        let mut handles = Vec::new();

        while self.queue.len() > 0 {
            let link = self.queue.pop().unwrap();
            if !self.links.contains(&link) {
                self.links.insert(link.clone());
                handles.push(Sitemap::follow(link));
            }
        }

        let mut results  = Vec::new();

        for handle in handles {
            results.push(handle.await.unwrap());
        }

        for links in results {
            for link in links {
                if !self.links.contains(&link) && !self.queue.contains(&link) {
                    self.queue.push(link);
                }
            }
        }

        if self.queue.len() > 0 {
            self.discover_queued_links().await;
        }
    }

    async fn follow(link: String) -> Result<Vec<String>, ()> {
        Ok(vec![String::from(link)])
    }

    fn is_external_link(&self, link: &str) -> bool {
        (link.contains("http") || link.contains("https")) && !link.contains(self.base_url)
    }

    fn is_internal_http_link(&self, link: &str) -> bool {
        !self.is_external_link(link)
        && !link.contains("mailto:")
        && !link.contains("tel:")
        && link != "#"
    }

    fn build_url(&self, link: &String) -> String {
        if !link.contains("http") {
            format!("{}{}", String::from(self.base_url), link)
        } else {
            link.to_string()
        }
    }

    /*fn follow(&mut self, link: &String) {
        let response = requests::get(self.build_url(link)).unwrap();
        let document = Html::parse_document(response.text().unwrap());
        let selector = Selector::parse("a").unwrap();

        let anchors = document.select(&selector);

        for link in anchors {
            if let Some(href) = link.value().attr("href") {
                if self.is_internal_http_link(href) {
                    let link_string = String::from(href);
                    if !self.links.contains(&link_string) {
                        self.links.insert(String::from(href));
                        self.follow(&String::from(href));
                    }
                }
            }
        }
   }*/
}
