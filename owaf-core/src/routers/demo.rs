use salvo::http::HeaderValue;
use salvo::prelude::*;
use salvo::proxy::Proxy;

use crate::{AppError, AppResult, db};


#[handler]
pub async fn hello(req: &mut Request, depot: &mut Depot, res: &mut Response) -> AppResult<()> {
    if !handle(req, depot, res).await {
        return Err(AppError::HttpStatus(StatusError::not_found()));
    }
    Ok(())
}

async fn handle(req: &mut Request, depot: &mut Depot, res: &mut Response) -> bool {
    let host_header = req.headers().get(salvo::http::header::HOST).and_then(|v| v.to_str().ok());
    let rest = req.param("rest").unwrap_or("").to_string();
    let query = req.uri().query().unwrap_or("").to_owned();
    if let Some(host) = host_header {
        let host_without_port = host.split(':').next().unwrap_or(host);
        println!("host: {} rest: {} query: {}", host_without_port, rest, query);
        let conn = db::pool();
        let message = format!("{} <{}>", host, rest);
        let _ = sqlx::query!(
            r#"
            INSERT INTO logs (message)
            VALUES ($1)
            "#,
            message,
        )
        .execute(conn)
        .await;
        let target: String;
        if host_without_port == "minio.example.com" {
            target = std::env::var("MINIO_URL").unwrap_or_else(|_| "http://localhost:9000".to_string());
            let host_val = target.strip_prefix("http://").unwrap_or(&target);
            let host_val = host_val.strip_prefix("https://").unwrap_or(host_val);
            req.headers_mut().insert(salvo::http::header::HOST, HeaderValue::from_str(host_val).unwrap());
        } else {
            return false;
        }
        let upstream = format!("{}/{}?{}", target, rest, query);
        println!("upstream: {}", upstream);
        let proxy = Proxy::use_reqwest_client(upstream);
        // Execute the proxy handler directly.
        use salvo::routing::FlowCtrl;
        use std::sync::Arc;
        let mut flow = FlowCtrl::new(vec![Arc::new(proxy)]);
        flow.call_next(req, depot, res).await;
        return true;
    }
    false
}
