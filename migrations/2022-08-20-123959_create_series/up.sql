CREATE TABLE IF NOT EXISTS series(
    name TEXT NOT NULL,
    tag TEXT NOT NULL,
    desc TEXT,
    img_file_path TEXT,

    PRIMARY KEY (tag)
);