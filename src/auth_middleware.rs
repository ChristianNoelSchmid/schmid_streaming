use chrono::{Utc, Duration, NaiveDateTime};
use dotenv::dotenv;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
};

use crate::{sqlite::get_conn, ip_middleware::get_ip};

pub struct AuthUser;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        struct IpAddr { last_log_on: NaiveDateTime }
        let ip = get_ip(req);

        // Load environment data from .env file in root directory
        dotenv().ok();
        let db = get_conn().await.unwrap();

        let result = sqlx::query_as!(IpAddr, "SELECT last_log_on FROM ip_addrs WHERE addr = ?", ip)
            .fetch_one(&db).await;

        return match result {
            Ok(res) => {
                return if Utc::now().naive_utc() - res.last_log_on < Duration::days(30) {
                    Outcome::Success(AuthUser)
                } else {
                    Outcome::Failure((Status::Unauthorized, ()))
                }
            },
            Err(sqlx::Error::RowNotFound) => Outcome::Failure((Status::Unauthorized, ())),
            Err(e) => panic!("{:?}", e)
        }
    }
}
