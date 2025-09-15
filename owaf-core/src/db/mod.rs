use std::sync::OnceLock;
use sqlx::SqlitePool;

use crate::config::DbConfig;

pub static SQLX_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub async fn init(config: &DbConfig) {
    let sqlx_pool = SqlitePool::connect(&config.url).await
        .expect("Database connection failed.");
    crate::db::SQLX_POOL
        .set(sqlx_pool)
        .expect("sqlx pool should be set")
}

pub fn pool() -> &'static SqlitePool {
    SQLX_POOL.get().expect("sqlx pool should be set")
}
