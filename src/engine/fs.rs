use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};

use bincode;
use shellexpand;

use crate::engine::asl;
use crate::engine::db;
use crate::engine::errors::QueryError;
use crate::config::config;
use crate::engine::db::DatabaseDefinition;
use crate::engine::pages::{ Item, Page, PAGE_SIZE };

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
    pub fn load_definitions(&self) -> Result<db::DatabaseDefinition, QueryError> {
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
    pub fn store_definitions(&self, db_definition: &DatabaseDefinition) -> Result<(), QueryError> {
        let mut file = fs::File::create(self.get_path(config::TABLE_DEFINITIONS_FILE))?;
        file.write_all(bincode::serialize(db_definition).unwrap().as_slice())?;
        Ok(())
    }

    /**
    * Create table files in the local filesystem
    */
    pub fn create_table_files(&self, table: &asl::Table) -> Result<(), QueryError> {
        fs::File::create(self.get_table_data_path(table))?;
        Ok(())
    }

    /**
    * Delete table files from the local filesystem
    */
    pub fn delete_table_files(&self, table: &asl::Table) -> Result<(), QueryError> {
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
    * Update the records that match the optionally provided conditions
    * Assumes that the types have already been validated
    */
    pub fn update_records(&self, table: &asl::Table, values: &Vec<asl::ColumnValue>,
                          condition: &Option<Box<asl::Expression>>) -> Result<(), QueryError> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(self.get_table_data_path(table))?;
        let mut page_buffer = [0; PAGE_SIZE];
        while file.read(&mut page_buffer)? > 0 {
            let page = Page::from_bytes(&page_buffer);
            for item in page.get_items() {
                let mut record = item.to_record(table);
                let record_needs_update = match condition {
                    Some(condition) => condition.evaluate_for_record(&table, &record)? == asl::Value::Bool(true),
                    None => true
                };
                if record_needs_update {
                    for value in values {
                        let column_index = table.columns.iter().position(|col| col == value.column).unwrap();
                        record.values[column_index] = value.value.clone();
                    }
                }
            }
        }
        Ok(())
    }

    /**
    * Insert a record into the last page of the table file
    * Creates a new page if the current one is full
    */
    pub fn insert_record(&self, table: &asl::Table, record: &asl::Record) -> Result<(), QueryError> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(self.get_table_data_path(table))?;
        let current_pages = file.metadata()?.len() / PAGE_SIZE as u64;
        let item = Item::from_record(record);
        let mut last_page_offset = if current_pages > 0 { PAGE_SIZE as u64 * (current_pages - 1) } else { 0 };
        let mut last_page;
        if current_pages > 0 {
            file.seek(SeekFrom::Start(last_page_offset))?;
            let mut page_bytes = [0u8; PAGE_SIZE];
            file.read(&mut page_bytes)?;
            last_page = Page::from_bytes(&page_bytes);
            match last_page.add_item(&item) {
                Ok(_) => (),
                Err(_) => {
                    let mut new_page = Page::new(last_page.id + 1);
                    new_page.add_item(&item)?;
                    last_page = new_page;
                    last_page_offset += PAGE_SIZE as u64;
                }
            }
        } else {
            let mut new_page = Page::new(1);
            new_page.add_item(&item)?;
            last_page = new_page;
        }
        file.seek(SeekFrom::Start(last_page_offset))?;
        file.write(&last_page.to_bytes())?;
        Ok(())
    }


    /**
    * Find records in the table file that match the given condition
    */
    pub fn select_records(&self, table: &asl::Table, columns: &Vec<String>,
                          condition: &Option<Box<asl::Expression>>) -> Result<Vec<asl::Record>, QueryError> {
        let mut file = fs::File::open(self.get_table_data_path(table))?;
        let mut page_buffer = [0; PAGE_SIZE];
        let mut records: Vec<asl::Record> = Vec::new();
        while file.read(&mut page_buffer)? > 0 {
            let page = Page::from_bytes(&page_buffer);
            for item in page.get_items() {
                let record = item.to_record(table);
                let include_record = match condition {
                    Some(condition) => condition.evaluate_for_record(&table, &record)? == asl::Value::Bool(true),
                    None => true
                };
                if include_record {
                    records.push(record);
                }
            }
        }
        Ok(records)
    }
}

