use std::sync::Arc;

use tokio::sync::Mutex;

use crate::app::app::App;

pub struct ProfileCommandHandler {}

impl ProfileCommandHandler {
    pub async fn handle(app: Arc<Mutex<App>>, action: String) -> String {
        let res = match action.as_str() {
            "ADD" => Self::add(app).await,
            "DEL" => Self::delete(app).await,
            "LS" => Self::list(app).await,
            _ => "Unknown action".to_string(),
        };

        res
    }

    async fn add(app: Arc<Mutex<App>>) -> String {
        let _lock = app.lock().await;
        "Profile added".to_string()
    }

    async fn delete(app: Arc<Mutex<App>>) -> String {
        let _lock = app.lock().await;
        "Profile deleted".to_string()
    }

    async fn list(app: Arc<Mutex<App>>) -> String {
        let app_lock = match app.try_lock() {
            Ok(a) => a,
            Err(e) => {
                panic!("{}", e);
            }
        };
        let mut res = "Profiles list:".to_string();
        for (key, value) in app_lock.profiles.iter() {
            res = format!("{}\n* {} : {}", res, key, value.private_key);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use dotenv::from_filename;
    use tokio_cron_scheduler::JobScheduler;

    use super::*;
    use crate::{
        app::app::App,
        profiles::config::Profile,
        rss::{config::RssConfig, rss::RssInstance},
    };

    #[test]
    fn test_socket_profile_add() {}

    #[test]
    fn test_socket_profile_del() {}

    #[tokio::test]
    async fn test_socket_profile_list() {
        from_filename(".env.test").ok();
        let rss_config = RssConfig { feeds: Vec::new() };

        let rss = RssInstance::new(rss_config).await;
        let scheduler = JobScheduler::new().await.unwrap();

        let mut profiles = HashMap::new();

        profiles.insert(
            "test".to_string(),
            Profile {
                ..Default::default()
            },
        );

        let app = App {
            rss,
            scheduler: Arc::new(scheduler),
            clients: HashMap::new(),
            profiles,
            feeds_jobs: HashMap::new(),
            feeds_map: HashMap::new(),
        };

        let result = ProfileCommandHandler::list(Arc::new(Mutex::new(app))).await;
        let expected = "Profiles list:\n* test : 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert_eq!(result, expected);
    }
}
