CREATE TABLE IF NOT EXISTS videos(
    id INT NOT NULL DEFAULT AUTO_INCREMENT, 

    name TEXT NOT NULL, 
    file_path TEXT NOT NULL, 
    file_format TEXT NOT NULL, 
    img_file_path TEXT, 
    desc TEXT,

    PRIMARY KEY (id)
);