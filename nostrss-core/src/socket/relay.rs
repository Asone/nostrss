use std::sync::Arc;

use tokio::sync::Mutex;

use crate::app::app::App;

pub struct RelayCommandHandler {}

impl RelayCommandHandler {
    pub async fn handle(app: Arc<Mutex<App>>, action: String) -> String {
        match action.as_str() {
            "ADD" => Self::add(app).await,
            "DEL" => Self::delete(app).await,
            "LS" => Self::list(app).await,
            _ => "Unknown action".to_string(),
        };

        "Relay handler".to_string()
    }

    async fn add(app: Arc<Mutex<App>>) -> String {
        let _lock = app.lock().await;
        "Relay added".to_string()
    }

    async fn delete(app: Arc<Mutex<App>>) -> String {
        let _lock = app.lock().await;
        "Relay deleted".to_string()
    }

    async fn list(app: Arc<Mutex<App>>) -> String {
        let _lock = app.lock().await;
        "Relays list".to_string()
    }
}
