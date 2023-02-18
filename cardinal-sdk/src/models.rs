use crate::schema::db_meta;
use crate::schema::dir_entrys;
use diesel::Insertable;

#[derive(Clone, Insertable)]
#[diesel(table_name = dir_entrys)]
pub struct DiskEntryRaw {
    pub the_path: Vec<u8>,
    pub the_meta: Vec<u8>,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = db_meta)]
pub struct DbMeta {
    pub the_key: Vec<u8>,
    pub the_value: Vec<u8>,
}
