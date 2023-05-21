table! {
    episodes (video_id) {
        video_id -> Integer,
        series_tag -> Text,
        season_idx -> Integer,
        idx -> Integer,
    }
}

table! {
    seasons (series_tag, idx) {
        series_tag -> Text,
        idx -> Integer,
        desc -> Nullable<Text>,
    }
}

table! {
    series (tag) {
        name -> Text,
        tag -> Text,
        desc -> Nullable<Text>,
        img_file_path -> Nullable<Text>,
    }
}

table! {
    videos (id) {
        id -> Integer,
        name -> Text,
        file_path -> Text,
        file_format -> Text,
        img_file_path -> Nullable<Text>,
        desc -> Nullable<Text>,
    }
}

joinable!(seasons -> series (series_tag));
joinable!(episodes -> videos (video_id));

allow_tables_to_appear_in_same_query!(episodes, seasons, series, videos,);
