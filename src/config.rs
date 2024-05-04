use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub video_dir_path: String
}