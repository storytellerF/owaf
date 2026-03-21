use salvo::cors::Cors;
use salvo::http::Method;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(FromRow, Serialize, Debug)]
pub struct Log {
    pub id: i64,
    pub message: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProxyConfig {
    #[serde(default)]
    pub proxy: HashMap<String, ProxyEntry>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProxyEntry {
    pub host: String,
    pub target: String,
}

static SQLX_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub fn pool() -> &'static SqlitePool {
    SQLX_POOL.get().expect("sqlx pool not initialized")
}

#[handler]
async fn get_logs() -> Result<Json<Vec<Log>>, StatusError> {
    // The logs table only has one column `message` originally, but rowid is implicit.
    let logs = sqlx::query_as!(
        Log,
        "SELECT rowid as id, message FROM logs ORDER BY rowid DESC LIMIT 100"
    )
    .fetch_all(pool())
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        StatusError::internal_server_error().brief(e.to_string())
    })?;
    Ok(Json(logs))
}

#[handler]
async fn get_proxy_config() -> Result<Json<Vec<ProxyEntry>>, StatusError> {
    // Read proxy.hcl from the workspace root (owaf-core/proxy.hcl) depending on execution context.
    let proxy_path =
        std::env::var("PROXY_CONFIG").unwrap_or_else(|_| "owaf-core/proxy.hcl".to_string());
    let proxy_config: ProxyConfig = if let Ok(content) = std::fs::read_to_string(&proxy_path) {
        hcl::from_str(&content).unwrap_or_else(|e| {
            tracing::error!("Failed to parse proxy config {}: {}", proxy_path, e);
            ProxyConfig {
                proxy: HashMap::new(),
            }
        })
    } else {
        tracing::warn!("Could not find proxy config at {}", proxy_path);
        ProxyConfig {
            proxy: HashMap::new(),
        }
    };

    let entries: Vec<ProxyEntry> = proxy_config.proxy.values().cloned().collect();
    Ok(Json(entries))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    // Determine the path to the sqlite database.
    let default_db = if std::path::Path::new("owaf-core/data/sqlx.sqlite").exists() {
        "sqlite:owaf-core/data/sqlx.sqlite"
    } else if std::path::Path::new("../owaf-core/data/sqlx.sqlite").exists() {
        "sqlite:../owaf-core/data/sqlx.sqlite"
    } else {
        "sqlite:owaf-core/data/sqlx.sqlite"
    };

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| default_db.to_string());
    tracing::info!("Connecting to database at {}", db_url);
    
    let sqlx_pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    SQLX_POOL.set(sqlx_pool).unwrap();

    let cors = Cors::new()
        .allow_origin("*")
        .allow_methods(vec![Method::GET, Method::POST, Method::DELETE, Method::PUT])
        .into_handler();

    let router = Router::new()
        .hoop(cors)
        .push(Router::with_path("api/log").get(get_logs))
        .push(Router::with_path("api/proxy-config").get(get_proxy_config));

    let listen_addr = "127.0.0.1:8009";
    tracing::info!("owaf-server listening on http://{}", listen_addr);
    let acceptor = TcpListener::new(listen_addr).bind().await;
    Server::new(acceptor).serve(router).await;
}
