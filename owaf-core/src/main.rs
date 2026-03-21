use salvo::catcher::Catcher;
use salvo::conn::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use salvo::server::ServerHandle;
use serde::Serialize;
use tokio::signal;

mod config;
mod db;
mod models;
mod utils;
mod hoops;
mod routers;

mod error;
pub use error::AppError;

pub type AppResult<T> = Result<T, AppError>;
pub type JsonResult<T> = Result<Json<T>, AppError>;
pub type EmptyResult = Result<Json<Empty>, AppError>;

pub fn json_ok<T>(data: T) -> JsonResult<T> {
    Ok(Json(data))
}
#[derive(Serialize, ToSchema, Clone, Copy, Debug)]
pub struct Empty {}
pub fn empty_ok() -> JsonResult<Empty> {
    Ok(Json(Empty {}))
}

#[tokio::main]
async fn main() {
    config::init();
    let config = crate::config::get();
    db::init(&config.db).await;

    let _guard = config.log.guard();
    tracing::info!("log level: {}", &config.log.filter_level);

    let service = Service::new(routers::root())
        .catcher(Catcher::default().hoop(hoops::error_404))
        .hoop(hoops::cors_hoop());
    println!("🔄 在以下位置监听 {}", &config.listen_addr);
    //Acme 支持，自动从 Let's Encrypt 获取 TLS 证书。例子请看 https://github.com/salvo-rs/salvo/blob/main/examples/acme-http01-quinn/src/main.rs
    if let Some(tls) = &config.tls {
        let listen_addr = &config.listen_addr;
        println!(
            "📖 Open API Page: https://{}/scalar",
            listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        println!(
            "🔑 Login Page: https://{}/login",
            listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        let config = RustlsConfig::new(Keycert::new().cert(tls.cert.clone()).key(tls.key.clone()));
        let acceptor = TcpListener::new(listen_addr).rustls(config).bind().await;
        let server = Server::new(acceptor);
        tokio::spawn(shutdown_signal(server.handle()));
        server.serve(service).await;
    } else {
        println!(
            "📖 Open API 页面: http://{}/scalar",
            config.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        println!(
            "🔑 Login Page: http://{}/login",
            config.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        let acceptor = TcpListener::new(&config.listen_addr).bind().await;
        let server = Server::new(acceptor);
        tokio::spawn(shutdown_signal(server.handle()));
        server.serve(service).await;
    }
}

async fn shutdown_signal(handle: ServerHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("ctrl_c signal received"),
        _ = terminate => tracing::info!("terminate signal received"),
    }
    handle.stop_graceful(std::time::Duration::from_secs(60));
}

#[cfg(test)]
mod tests {
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};

    use crate::config;

    static INIT: std::sync::Once = std::sync::Once::new();

    pub fn setup_test_env() {
        INIT.call_once(|| {
            std::env::set_var("APP_LISTEN_ADDR", "127.0.0.1:0");
            std::env::set_var("DATABASE_URL", "sqlite::memory:");
            crate::config::init();
        });
    }

    #[tokio::test]
    async fn test_hello_world() {
        setup_test_env();

        let service = Service::new(crate::routers::root());

        let content = TestClient::get(format!(
            "http://{}",
            config::get().listen_addr.replace("0.0.0.0", "127.0.0.1")
        ))
        .send(&service)
        .await
        .take_string()
        .await
        .unwrap();
        assert_eq!(content, "Hello World from salvo");
    }

    #[tokio::test]
    async fn test_minio_proxy() {
        use std::env;
        use testcontainers::{runners::AsyncRunner, ImageExt};
        use testcontainers_modules::minio::MinIO;
        
        setup_test_env();
        
        // Note: db::init sets SQLX_POOL. If test_hello_world runs concurrently, it will fail if it tries to init db without url.
        // We will initialize db pool here (using OnceLock properties).
        if crate::db::SQLX_POOL.get().is_none() {
            let db_config = &crate::config::get().db;
            let sqlx_pool = sqlx::SqlitePool::connect(&db_config.url).await.unwrap();
            if crate::db::SQLX_POOL.set(sqlx_pool).is_ok() {
                // run migrations for the sqlite memory db
                let pool = crate::db::pool();
                sqlx::migrate!("./migrations").run(pool).await.unwrap();
            }
        }
        
        // Start testcontainer minio
        let minio_image = MinIO::default();
        let node = minio_image.start().await.unwrap();
        let host_port = node.get_host_port_ipv4(9000).await.unwrap();
        let minio_url = format!("http://127.0.0.1:{}", host_port);
        env::set_var("MINIO_URL", &minio_url);

        let service = Service::new(crate::routers::root());

        let res = TestClient::get("http://minio.example.com/some/path")
            .add_header("Host", "minio.example.com", true)
            .send(&service)
            .await;

        assert!(res.status_code.is_some());
        let status = res.status_code.unwrap();
        // Since it's a proxy reaching MinIO without auth, it will normally return 403.
        // If the proxy fails to reach it, it would be 502 or something.
        assert_eq!(status, salvo::http::StatusCode::FORBIDDEN);
    }
}
