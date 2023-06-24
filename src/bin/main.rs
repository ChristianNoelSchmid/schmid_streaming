#[macro_use] extern crate rocket;

use std::env;

use chrono::{Utc, NaiveDateTime, Duration};

use dotenv::dotenv;
use rocket_dyn_templates::{Template, context};
use rocket_seek_stream::SeekStream;

use rocket::{
    fs::FileServer, response::Redirect, form::Form
};

use schmid_streaming::{sqlite::get_conn, auth_middleware::AuthUser, ip_middleware::Ip};
use serde::Serialize;

#[derive(FromForm)]
struct SignInForm {
    access_key: String
}

#[get("/sign-in")]
fn sign_in_get() -> Template {
    Template::render("sign-in", context! {})
}

#[post("/sign-in", data = "<input>")]
async fn sign_in_post(input: Form<SignInForm>, ip: Ip) -> Redirect {

    struct IpAddr { pub id: i64, pub last_log_on: NaiveDateTime }

    // Load environment data from .env file in root directory
    dotenv().ok();
    let secret = env::var("SECRET").expect("SECRET must be set");
    let db = get_conn().await.expect("Could not connect to database");

    let ip_addr = sqlx::query_as!(IpAddr, "SELECT id, last_log_on FROM ip_addrs WHERE addr = ?", ip.0).fetch_one(&db).await;

    return if input.access_key == secret {
        let utc_now = Utc::now().naive_utc();
        match ip_addr {
            Ok(ip_addr) => {
                if Utc::now().naive_utc() - ip_addr.last_log_on < Duration::days(30) {
                    sqlx::query!(
                        "UPDATE ip_addrs SET last_log_on = ? WHERE id = ?",
                        utc_now, ip_addr.id
                    ).execute(&db).await.unwrap();
                }
            },
            Err(sqlx::Error::RowNotFound) => {
                sqlx::query!(
                    "INSERT INTO ip_addrs (addr, last_log_on) VALUES (?, ?)",
                    ip.0, utc_now,
                ).execute(&db).await.unwrap();
            },
            Err(e) => panic!("{:?}", e)
        }
        Redirect::to(uri!(index))
    } else {
        Redirect::to(uri!(sign_in_get))
    };
}

#[catch(401)]
fn redirect_to_sign_in() -> Redirect {
    Redirect::to(uri!(sign_in_get))
}

#[get("/")]
async fn index(_user: AuthUser) -> Template {

    #[derive(Serialize)]
    pub struct Movie { pub id: String, pub name: String, pub img_file_path: Option<String> } 
    #[derive(Serialize)]
    pub struct Series { pub tag: String, pub name: String, pub img_file_path: Option<String> }
    #[derive(Serialize)]
    pub struct IndexViewModel { pub movies: Vec<Movie>, pub series: Vec<Series>, }

    let db = get_conn().await.unwrap();

    let movies = sqlx::query_as!(Movie, r"
        SELECT id, name, img_file_path FROM videos v WHERE NOT EXISTS (
            SELECT * FROM episodes WHERE video_id = v.id
        )"
    )
        .fetch_all(&db).await.unwrap();

    let srs = sqlx::query_as!(Series, "SELECT tag, name, img_file_path FROM series")
        .fetch_all(&db).await.unwrap();       

    Template::render(
        "index",
        context! { 
            movies: movies,
            series: srs
        },
    )
}

#[get("/series/<tg>")]
async fn series_details(tg: String, _user: AuthUser) -> Template {

    struct Series { name: String, desc: Option<String>, img_file_path: Option<String>, }
    struct Season { idx: i64 }

    #[derive(Serialize)]
    struct Episode { video_id: String, name: String }

    let db = get_conn().await.unwrap();

    let srs = sqlx::query_as!(Series,
        "SELECT name, desc, img_file_path FROM series WHERE tag = ?", tg)
        .fetch_one(&db).await.unwrap();

    let seasons = sqlx::query_as!(Season, 
        "SELECT idx FROM seasons WHERE series_tag = ? ORDER BY idx", tg)
        .fetch_all(&db).await.unwrap();

    let mut season_episodes = Vec::new();
    for season in seasons {
        let eps = sqlx::query_as!(Episode, r"
                SELECT video_id, name FROM episodes JOIN videos ON video_id = id
                WHERE series_tag = ? AND season_idx = ?
                ORDER BY idx
            ",
            tg, season.idx
        )
            .fetch_all(&db).await.unwrap();

        season_episodes.push(eps);
    }
    Template::render("series_details", context! { 
        series_name: srs.name,
        series_desc: srs.desc,
        series_img_path: srs.img_file_path,
        season_episodes: season_episodes
    })
}

#[get("/watch/<video_id>")]
async fn watch<'a>(video_id: String) -> std::io::Result<SeekStream<'a>> {

    struct Video { file_path: String }

    let db = get_conn().await.unwrap();
    let video = sqlx::query_as!(Video, "SELECT file_path FROM videos WHERE id = ?", video_id)
        .fetch_one(&db).await.unwrap();

    SeekStream::from_path(format!("./static/videos/{}", video.file_path))
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    rocket::build()
        .attach(Template::fairing())
        .register("/", catchers![redirect_to_sign_in])
        .mount("/public", FileServer::from("./static"))
        .mount("/", routes!(sign_in_get, sign_in_post, index, series_details, watch))
}