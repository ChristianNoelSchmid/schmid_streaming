use diesel::Queryable;
use serde::Serialize;
///
/// A single video, with it's corresponding data,
/// and the path to the actual video file.
///
#[derive(Debug, Queryable, Serialize)]
pub struct Video {
    /// The Video's primary key
    pub id: i32,
    /// The name of the Video
    pub name: String,
    /// The path to the Video's file
    pub file_path: String,
    /// The format the Video file uses
    pub file_format: String,
    /// The file path to the thumbnail image for the Video
    pub img_file_path: Option<String>,
    /// A description of the Video
    pub desc: Option<String>,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Episode {
    pub video_id: i32,
    pub series_tag: String,
    pub season_idx: i32,
    pub idx: i32,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Season {
    pub series_tag: String,
    pub idx: i32,
    pub desc: Option<String>,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Series {
    pub name: String,
    pub tag: String,
    pub desc: Option<String>,
    pub img_file_path: Option<String>,
}
