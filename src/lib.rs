use requests;
use scraper::{Html, Selector};
use std::collections::HashSet;

pub fn run() {
    println!("Hello Mapmaker");
    let sitemap = Sitemap::new("https://www.test.de");
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

        sitemap.follow(&String::from(base_url));
        sitemap
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

    fn follow(&mut self, link: &String) {
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
   }
}
