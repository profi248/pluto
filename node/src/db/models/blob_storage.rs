use diesel::prelude::*;
use crate::db::{ Database, DatabaseError };
use crate::db::schema::blob_storage;

#[derive(Queryable, Debug, Identifiable, Insertable)]
#[primary_key(key)]
#[table_name = "blob_storage"]
pub struct BlobStorage {
    pub key: String,
    pub value: Vec<u8>
}

impl Database {
    pub fn get_by_key(key: String) -> QueryResult<Option<Vec<u8>>> {
        blob_storage::table.find(key)
            .select(blob_storage::value)
            .first(&Self::connect())
            .optional()
    }

    pub fn set_by_key(key: String, value: Vec<u8>) -> QueryResult<()> {
        let blob = BlobStorage { key: key.clone(), value: value.clone() };
        match Self::get_by_key(key.clone())? {
            Some(_) => {
                diesel::update(blob_storage::table)
                    .filter(blob_storage::key.eq(key))
                    .set(blob_storage::value.eq(value))
                    .execute(&Self::connect()).map(|_| ())
            },
            None => {
                diesel::insert_into(blob_storage::table)
                    .values(&blob)
                    .execute(&Self::connect()).map(|_| ())
            }
        }
    }
}
