#[macro_use] extern crate rocket;

use diesel::{
    dsl::{exists, not},
    prelude::*,
};

use dotenv::dotenv;
use rocket_dyn_templates::{Template, context};
use rocket_seek_stream::SeekStream;

use rocket::{
    fs::FileServer
};

use schmid_streaming::{models::*, sqlite::get_conn};
use serde::Serialize;

#[get("/")]
fn index() -> Template {
    use schmid_streaming::schema::{
        episodes::dsl::*,
        series::{dsl::*},
        videos::{self, dsl::*},
    };
    #[derive(Serialize)]
    pub struct IndexViewModel {
        pub movies: Vec<Video>,
        pub series: Vec<Series>,
    }

    let db = get_conn();

    let movies = videos
        .filter(not(exists(episodes.find(videos::id))))
        .load::<Video>(&db)
        .expect("could not run query to find movies.");

    let srs = series.load::<Series>(&db)
        .expect("could not run query to find all series");

    Template::render(
        "index",
        context! { 
            movies: movies,
            series: srs
        },
    )
}

#[get("/series/<tg>")]
fn series_details(tg: String) -> Template {
    use schmid_streaming::schema::{
        episodes::{self, dsl::*},
        seasons::{self, dsl::*},
        series::dsl::*,
        videos::{self, dsl::*},
    };
    #[derive(Serialize)]
    struct SeriesDetailsViewModel {
        series_name: String,
        series_desc: Option<String>,
        series_img_path: Option<String>,
        seasons_ids_and_names: Vec<Vec<(i32, String)>>,
    }

    let db = get_conn();

    let srs = series
        .find(&tg)
        .first::<Series>(&db)
        .expect("Could not find series with tag");
    let series_name = srs.name;
    let series_desc = srs.desc;
    let series_img_path = srs.img_file_path;

    let series_seasons = seasons
        .filter(seasons::series_tag.eq(&tg))
        .order_by(seasons::idx)
        .load::<Season>(&db)
        .expect("could not load seasons for series");

    let mut seasons_ids_and_names = Vec::new();
    for season in series_seasons {
        let eps = episodes
            .filter(episodes::series_tag.eq(&tg).and(season_idx.eq(season.idx)))
            .inner_join(videos)
            .order_by(episodes::idx)
            .select((video_id, videos::name))
            .load::<(i32, String)>(&db)
            .expect("could not load episode for season");

        seasons_ids_and_names.push(eps);
    }
    Template::render("series_details", context! { 
        series_name: series_name,
        series_desc: series_desc,
        series_img_path: series_img_path,
        seasons_ids_and_names: seasons_ids_and_names
    })
}

#[get("/watch/<video_id>")]
fn watch<'a>(video_id: i32) -> std::io::Result<SeekStream<'a>> {
    use schmid_streaming::schema::
        videos::dsl::*
    ;

    let db = get_conn();
    let video = videos.find(video_id).first::<Video>(&db).expect("Couldn't find video");
    println!("{}", format!("./static/{}", video.file_path));
    SeekStream::from_path(format!("./static/videos/{}", video.file_path))
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build()
        .attach(Template::fairing())
        .mount("/public", FileServer::from("./static"))
        .mount("/", routes!(index, series_details, watch))
}