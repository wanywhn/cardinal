-- Your SQL goes here
CREATE TABLE IF NOT EXISTS dir_entrys
(
    the_path BLOB PRIMARY KEY NOT NULL,
    the_meta BLOB NOT NULL
) WITHOUT ROWID;
