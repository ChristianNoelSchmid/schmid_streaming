use std::convert::Infallible;

use rocket::{request::{FromRequest, Outcome}};

pub struct Ip(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Ip {
    type Error = Infallible;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(Ip(get_ip(req)))
    }
}

pub fn get_ip<'r>(req: &'r rocket::Request<'_>) -> String {
    let ip = req.headers().get_one("X-Forwarded-For")
        .and_then(|fwd| Some(fwd.to_string()))
        .unwrap_or(req.remote().unwrap().ip().to_string());

    ip
}