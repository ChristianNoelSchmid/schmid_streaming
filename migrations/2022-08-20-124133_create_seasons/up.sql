CREATE TABLE IF NOT EXISTS seasons(
    series_tag TEXT NOT NULL,
    idx INT NOT NULL,
    desc TEXT,

    PRIMARY KEY (series_tag, idx),
    FOREIGN KEY (series_tag) REFERENCES series (tag)
);