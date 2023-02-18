// @generated automatically by Diesel CLI.

diesel::table! {
    db_meta (the_key) {
        the_key -> Binary,
        the_value -> Binary,
    }
}

diesel::table! {
    dir_entrys (the_path) {
        the_path -> Binary,
        the_meta -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(db_meta, dir_entrys,);
