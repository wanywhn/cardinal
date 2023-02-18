// @generated automatically by Diesel CLI.

diesel::table! {
    dir_entrys (the_path) {
        the_path -> Binary,
        the_meta -> Binary,
    }
}
