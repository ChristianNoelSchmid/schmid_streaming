use std::env;
use sqlx::SqlitePool;

pub async fn get_conn() -> sqlx::Result<SqlitePool> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in env vars");
    SqlitePool::connect(&database_url).await
}
