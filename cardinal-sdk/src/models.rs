use crate::schema::dir_entrys;
use diesel::Insertable;

#[derive(Clone, Insertable)]
#[diesel(table_name = dir_entrys)]
pub struct DiskEntryRaw {
    pub the_path: Vec<u8>,
    pub the_meta: Vec<u8>,
}
