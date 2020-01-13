use std::fs;

use shellexpand;
use crate::config::config;
use crate::engine::model;

/**
Ensure that the db path exists
*/
pub fn ensure_db_path() -> std::io::Result<()> {
    fs::create_dir_all(&shellexpand::tilde(&config::DB_PATH).as_ref())?;
    Ok(())
}

pub fn store_database_definition(database_definition: model::DatabaseDefinition) -> std::io::Result<()> {
    Ok(())
}