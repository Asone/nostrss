use crate::rss::config::Feed;
use feed_rs::model::Entry;
use log::error;
use new_string_template::{error::TemplateError, template::Template};
use std::env;
use std::{collections::HashMap, fs};

#[derive(Debug)]
pub enum TemplateParserError {
    LoadError,
}

/// Provides template rendering to the application
pub struct TemplateProcessor {}

impl TemplateProcessor {
    // Tries to load a template through an option path
    // If template is not provided, the method fallsback
    // to environment template.
    fn load_template(path: Option<String>) -> Result<String, TemplateParserError> {
        match path {
            Some(path) => {
                let file = fs::read_to_string(path);
                match file {
                    Ok(file_content) => Ok(file_content),
                    Err(e) => {
                        error!("{}", e);
                        Err(TemplateParserError::LoadError)
                    }
                }
            }
            None => Ok(Self::get_default_env_template()),
        }
    }

    // Parses template from environment
    fn get_default_env_template() -> String {
        match env::var("DEFAULT_TEMPLATE") {
            Ok(val) => val,
            Err(e) => {
                error!("{}", e);
                panic!();
            }
        }
    }

    // Parses template with data
    pub fn parse(data: Feed, entry: Entry) -> Result<String, TemplateError> {
        let template = Self::load_template(data.clone().template).unwrap();
        let mut map = Self::parse_entry_to_hashmap(entry);

        map.insert("name", data.name.clone());

        let mut tags_string = "".to_string();

        for tag in data.tags.unwrap_or(Vec::new()) {
            tags_string = format!("{} #{}", tags_string, tag);
        }

        map.insert("tags", tags_string.trim().to_string());

        let templ = Template::new(template);

        templ.render(&map)
    }

    // creates a HashMap from the entry data
    // The HashMap is currently consumed by the template engine
    fn parse_entry_to_hashmap(data: Entry) -> HashMap<&'static str, String> {
        let mut map = HashMap::new();

        let title = match data.title {
            Some(title) => title.content,
            None => "".to_string(),
        };

        map.insert("title", title);
        map.insert("url", data.links[0].clone().href);

        let summary = match data.summary {
            Some(summary) => summary.content,
            None => "".to_string(),
        };

        map.insert("summary", summary);

        // published time
        if data.published.is_some() {
            map.insert("published",data.published.unwrap().to_string());
        }

        if data.content.is_some() {
            let content = data.content.unwrap();

            if content.body.is_some() {
                let body = content.body.unwrap();
                map.insert("content",body);
            }
        }
        
        // authors 
        if data.authors.len() > 0 {
            let authors = data.authors.into_iter().map(|author|{
                author.name
            }).collect();

            map.insert("author",authors);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    extern crate mime;

    use super::*;
    use dotenv::from_filename;

    use feed_rs::model::{Content, Link, Text};

    #[test]
    fn test_default_template_fallback() {
        from_filename(".env.test").ok();

        let entry = Entry {
            content: Some(Content {
                body: Some("Test".to_string()),
                ..Default::default()
            }),
            links: [Link {
                href: "https://www.nostr.info".to_string(),
                rel: None,
                media_type: None,
                href_lang: None,
                title: None,
                length: None,
            }]
            .to_vec(),
            ..Default::default()
        };

        let feed = Feed {
            tags: Some(Vec::new()),
            ..Default::default()
        };

        let result = TemplateProcessor::parse(feed, entry);

        assert_eq!(result.is_ok(), true);

        let expected =
            "test nostrss template\nFeed: Generic feed\nUrl: https://www.nostr.info\nTags: "
                .to_string();
        let result = result.unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_custom_template() {
        from_filename(".env.test").ok();

        let entry = Entry {
            title: Some(Text {
                content_type: mime::TEXT_PLAIN,
                src: None,
                content: "Test content".to_string(),
            }),
            content: Some(Content {
                body: Some("Test body".to_string()),
                ..Default::default()
            }),
            links: [Link {
                href: "https://www.nostr.info".to_string(),
                rel: None,
                media_type: None,
                href_lang: None,
                title: None,
                length: None,
            }]
            .to_vec(),
            ..Default::default()
        };

        let tags = ["Test".to_string(), "nostrss".to_string()].to_vec();
        let feed = Feed {
            tags: Some(tags),
            template: Some("./src/fixtures/default.template".to_string()),
            ..Default::default()
        };

        let result = TemplateProcessor::parse(feed, entry);

        assert_eq!(result.is_ok(), true);

        let result = result.unwrap();
        let expected = "Default nostrss template file\nFeed: Generic feed\nUrl: https://www.nostr.info\nTags: #Test #nostrss".to_string();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_entry_to_hashmap() {
        from_filename(".env.test").ok();
        let entry = Entry {
            title: Some(Text {
                content_type: mime::TEXT_PLAIN,
                src: None,
                content: "Test content".to_string(),
            }),
            content: Some(Content {
                body: Some("Test body".to_string()),
                ..Default::default()
            }),
            links: [Link {
                href: "https://www.nostr.info".to_string(),
                rel: None,
                media_type: None,
                href_lang: None,
                title: None,
                length: None,
            }]
            .to_vec(),
            ..Default::default()
        };

        let hashmap = TemplateProcessor::parse_entry_to_hashmap(entry);

        assert_eq!(hashmap["title"], "Test content");
        assert_eq!(hashmap["url"], "https://www.nostr.info");
    }

    #[test]
    fn test_template_loading() {
        let path = "./src/fixtures/default.template".to_string();
        let result = TemplateProcessor::load_template(Some(path));
        assert_eq!(result.is_ok(), true);

        let result = result.unwrap();
        let expected =
            "Default nostrss template file\nFeed: {name}\nUrl: {url}\nTags: {tags}".to_string();
        assert_eq!(result, expected);

        let bad_path = "./src/fixture/nonexistant.template".to_string();
        let result = TemplateProcessor::load_template(Some(bad_path));
        assert_eq!(result.is_err(), true);

        from_filename(".env.test").ok();
        let result = TemplateProcessor::load_template(None);
        assert_eq!(result.is_ok(), true);

        let result = result.unwrap();
        let expected = "test nostrss template\nFeed: {name}\nUrl: {url}\nTags: {tags}".to_string();
        assert_eq!(result, expected);
    }
}
