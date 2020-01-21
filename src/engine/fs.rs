use std::fs;
use std::io::{Read, Write};

use bincode;
use shellexpand;

use crate::engine::asl;
use crate::engine::db;
use crate::config::config;
use crate::engine::db::DatabaseDefinition;


const PAGE_SIZE : usize = 8 * 1024;
const U32_SIZE_BYTES : i32 = 4;

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
    pub fn insert_record(&self, table: &asl::Table, values: Vec<asl::Value>) -> std::io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(self.get_table_data_path(table))?;
        for value in values {
            let value_bytes = value.to_be_bytes();
            let mut value_bytes_with_length: Vec<u8> = Vec::new();
            let value_length_bytes = (value_bytes.len() as i32).to_be_bytes().to_vec();
            for byte in value_length_bytes {
                value_bytes_with_length.push(byte);
            }
            for byte in value_bytes {
                value_bytes_with_length.push(byte);
            }
            file.write(&value_bytes_with_length)?;
        }
        Ok(())
    }


    /**
    * Find records in the table file that match the given condition
    */
    pub fn select_records(&self, table: &asl::Table, columns: &Vec<String>,
                          condition: &Option<Box<asl::Expression>>) -> std::io::Result<Vec<Vec<asl::Value>>> {
        let mut file = fs::File::open(self.get_table_data_path(table))?;
        let mut buffer = [0; PAGE_SIZE];
        let mut records: Vec<Vec<asl::Value>> = Vec::new();
        let mut current_record: Vec<asl::Value> = Vec::new();
        let mut current_token: &[u8];
        let mut carryover_bytes = Vec::new();
        let mut current_target_bytes: i32 = U32_SIZE_BYTES;
        let mut reading_size = true;
        let mut carryover = false;
        while file.read(&mut buffer)? > 0 {
            let mut offset = 0;
            while offset < buffer.len() {
                if offset + current_target_bytes as usize > buffer.len() {
                    carryover = true;
                    let remaining_buffer_bytes = &buffer[offset..buffer.len()];
                    carryover_bytes.extend_from_slice(remaining_buffer_bytes);
                    current_target_bytes -= remaining_buffer_bytes.len() as i32;
                    break;
                }
                if carryover {
                    let remaining_token_bytes = &buffer[offset..offset + current_target_bytes as usize];
                    carryover_bytes.extend_from_slice(remaining_token_bytes);
                    current_token = carryover_bytes.as_slice();
                } else {
                    current_token = &buffer[offset..offset + current_target_bytes as usize];
                }
                if current_token.len() == 0 {
                    break;  // there are no more records in this buffer
                }

                offset += current_target_bytes as usize;
                if reading_size {
                    let mut size_bytes: [u8; 4] = Default::default();
                    size_bytes.copy_from_slice(&current_token[0..4]);
                    current_target_bytes = i32::from_be_bytes(size_bytes);
                    reading_size = false;
                } else {
                    let value_type = &table.columns[current_record.len()].column_type;
                    current_record.push(
                        asl::Value::from_be_bytes(
                            Vec::from(current_token),
                            value_type
                        )
                    );
                    if current_record.len() == table.columns.len() {
                        records.push(current_record);
                        current_record = Vec::new();
                    }
                    reading_size = true;
                    current_target_bytes = U32_SIZE_BYTES;
                }
                carryover = false;
                carryover_bytes = Vec::new();
            }
        }
        Ok(records)
    }
}

