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
    async fn load_template(path: Option<String>) -> Result<String, TemplateParserError> {
        match path {
            Some(path) => {
                let file = fs::read_to_string(&path);
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
                panic!("");
            }
        }
    }

    // Parses template with data
    pub async fn parse(data: Feed, entry: Entry) -> Result<String, TemplateError> {
        let template = Self::load_template(data.clone().template).await.unwrap();
        let mut map = Self::parse_entry_to_hashmap(entry);

        map.insert("name", data.name.clone());

        let mut tags_string = "".to_string();
        for tag in data.clone().tags.unwrap() {
            // tags.push(Tag::Hashtag(tag.clone()));
            tags_string = format!("{} #{}", tags_string, tag);
        }

        let templ = Template::new(&template);

        templ.render(&map)
    }

    // created a HashMap from the entry data
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

        map
    }
}

mod tests {
    use dotenv::from_filename;
    use feed_rs::model::Content;

    use super::*;

    #[test]
    fn test_default_template_fallback() {
        from_filename(".env.test").ok();

        let entry = Entry {
            content: Some(Content {
                ..Default::default()
            }),

            ..Default::default()
        };
    }

    fn test_custom_template() {
        from_filename(".env.test").ok();
        let entry = Entry {
            ..Default::default()
        };
    }

    fn test_entry_to_hashmap() {
        from_filename(".env.test").ok();
        let entry = Entry {
            ..Default::default()
        };
        let hashmap = TemplateProcessor::parse_entry_to_hashmap(entry);

        assert_eq!(hashmap["title"], "test template")
    }

    fn test_template_loading() {}
}
