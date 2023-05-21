CREATE TABLE IF NOT EXISTS episodes(
    video_id INT NOT NULL,

    series_tag TEXT NOT NULL,
    season_idx INT NOT NULL,
    idx INT NOT NULL,

    PRIMARY KEY (video_id),
    FOREIGN KEY (video_id) REFERENCES video(id),
    FOREIGN KEY (series_tag, season_idx) REFERENCES season (series_tag, idx)
);