use lazy_static::lazy_static;
use std::env;

use dotenv::dotenv;
use regex::Regex;
use rocket::{
    http::{Cookie, Status},
    request::{FromRequest, Outcome},
};

lazy_static! {
    static ref AUTH_RE: Regex =
        Regex::new(r"(?i)basic\s*(?-i)(?P<tok>[^;]+)(;(?P<user_id>\d+))?").unwrap();
}

pub struct AuthUser;

#[derive(Debug)]
pub struct AuthUserError;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = AuthUserError;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        dotenv().ok();
        let secret = env::var("SECRET").expect("SECRET must be set");
        let key = req.query_value::<String>("secret");
        let mut found_id = false;

        if let Some(Ok(key)) = key {
            if key == secret {
                found_id = true;
            }
        }

        if let Some(cookie) = req.cookies().get("secret") {
            if cookie.value() == secret {
                found_id = true;
            }
        }

        if found_id {
            return Outcome::Success(AuthUser);
        }

        Outcome::Error((Status::Unauthorized, AuthUserError))
    }
}
