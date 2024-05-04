#[macro_use] extern crate rocket;

use std::{env, fs, path::PathBuf, str::FromStr};

use rocket_dyn_templates::{Template, context};
use rocket_seek_stream::SeekStream;

use rocket::{ form::Form, fs::FileServer, http::{Cookie, CookieJar}, response::Redirect, State };

use schmid_streaming::{auth_middleware::AuthUser, config::Config, sqlite::get_conn};
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
async fn sign_in_post(input: Form<SignInForm>, jar: &CookieJar<'_>) -> Redirect {
    dotenv::dotenv().unwrap();
    let secret = env::var("SECRET").expect("SECRET must be set");

    return if input.access_key == secret {
        let cookie = Cookie::build(("secret", secret))
            .path("/")
            .secure(true)
            .http_only(true)
            .permanent()
            .build();

        jar.add(cookie);

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
        ) AND active = TRUE"
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
                WHERE series_tag = ? AND season_idx = ? AND active = TRUE
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
async fn watch<'a>(video_id: String, config: &State<Config>) -> std::io::Result<SeekStream<'a>> {

    struct Video { file_path: String }

    let db = get_conn().await.unwrap();
    let video = sqlx::query_as!(Video, "SELECT file_path FROM videos WHERE id = ?", video_id)
        .fetch_one(&db).await.unwrap();

    let mut path = PathBuf::from_str(&config.video_dir_path).unwrap();
    path.push(video.file_path);
    SeekStream::from_path(path.to_str().unwrap())
}

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();
    let config = serde_json::from_str::<Config>(&fs::read_to_string("./config.json").unwrap()).unwrap();
    rocket::build()
        .attach(Template::fairing())
        .manage(config)
        .register("/", catchers![redirect_to_sign_in])
        .mount("/public", FileServer::from("./static"))
        .mount("/", routes!(sign_in_get, sign_in_post, index, series_details, watch))
}