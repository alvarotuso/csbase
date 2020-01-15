use std::fs;
use std::io::{Read, Write};

use bincode;
use shellexpand;

use crate::engine::asl;
use crate::engine::db;
use crate::config::config;
use crate::engine::db::DatabaseDefinition;


#[derive(Debug)]
pub struct DBFileSystem {
    base_path: String,
}

impl DBFileSystem {
    pub fn new() -> DBFileSystem {
        DBFileSystem { base_path: shellexpand::tilde(&config::DB_PATH).to_string() }
    }

    fn get_path(&self, path: &str) -> String {
        format!("{}/{}", self.base_path, path)
    }

    fn get_table_data_path(&self, table: &asl::Table) -> String {
        self.get_path(&format!("{}_data.csbase", table.name))
    }

    /**
    * Read and deserialize the database definition file
    */
    pub fn load_definitions(&self) -> std::io::Result<db::DatabaseDefinition> {
        let mut file = fs::File::open(self.get_path(config::TABLE_DEFINITIONS_FILE))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let db_definition: DatabaseDefinition = bincode::deserialize(buffer.as_slice()).unwrap();
        print!("Loaded db definition: {:#?}", db_definition);
        Ok(db_definition)
    }

    /**
    * Serialize and store the database definition
    */
    pub fn store_definitions(&self, db_definition: &DatabaseDefinition) -> std::io::Result<()> {
        let mut file = fs::File::create(self.get_path(config::TABLE_DEFINITIONS_FILE))?;
        file.write_all(bincode::serialize(db_definition).unwrap().as_slice())?;
        Ok(())
    }

    /**
    * Create table files in the local filesystem
    */
    pub fn create_table_files(&self, table: &asl::Table) -> std::io::Result<()> {
        fs::File::create(self.get_table_data_path(table))?;
        Ok(())
    }

    /**
    * Delete table files from the local filesystem
    */
    pub fn delete_table_files(&self, table: &asl::Table) -> std::io::Result<()> {
        fs::remove_file(self.get_table_data_path(table))?;
        Ok(())
    }

    /**
    * Ensure that the database path exists
    */
    pub fn ensure_base_path(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_path)?;
        Ok(())
    }

    /**
    * Insert a record into the table file
    */
    pub fn insert_record(&self, table: &asl::Table) {

    }

    /**
    * Update a record from the table file
    */
    pub fn update_record(&self, table: &asl::Table) {

    }

    /**
    * Delete a record from the table file
    */
    pub fn delete_record(&self, table: &asl::Table) {

    }
}

