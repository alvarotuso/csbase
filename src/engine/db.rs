use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::engine::asl;
use crate::engine::errors::{QueryError, SystemError};
use crate::engine::fs::DBFileSystem;
use crate::sql_grammar;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DatabaseDefinition {
    tables: HashMap<String, asl::Table>,
}

#[derive(Debug)]
pub struct Database {
    db_definition: DatabaseDefinition,
    db_filesystem: DBFileSystem,
}

impl Database {
    pub fn new() -> Database {
        Database {
            db_definition: DatabaseDefinition { tables: HashMap::new() },
            db_filesystem: DBFileSystem::new(),
        }
    }

    pub fn bootstrap(&mut self) -> Result<(), SystemError> {
        self.db_filesystem.ensure_base_path()?;
        match self.db_filesystem.load_definitions() {
            Ok(definition) => {
                self.db_definition = definition
            },
            Err(_) => {}
        }
        Ok(())
    }

    /**
    * Get an existing table from the definition
    */
    fn get_table(&self, table_name: &str) -> Result<&asl::Table, QueryError> {
        match self.db_definition.tables.get(table_name) {
            Some(table) => Ok(table),
            None => Err(QueryError::NotFound(String::from(table_name)))
        }
    }

    fn validate_select(&self, query: asl::SelectQuery) -> Result<(), QueryError> {
        Ok(())
    }

    fn run_select(&self, query: asl::SelectQuery) -> Result<String, QueryError> {
        let table = self.get_table(&query.table)?;
        let records = self.db_filesystem.select_records(table,
                                                        &query.columns, &query.condition)?;
        Ok(format!("Found Records {:?}", records))
    }

    fn validate_insert(&self, table: &asl::Table,
                       query: &asl::InsertQuery,
                       evaluated_expressions: &Vec<asl::Value>) -> Result<(), QueryError> {
        if query.columns.len() != evaluated_expressions.len() {
            return Err(
                QueryError::ValidationError(
                    String::from("The number of columns and values doesn't match")))
        }
        for (idx, column_name) in query.columns.iter().enumerate() {
            let value = &evaluated_expressions[idx];
            let table_column = match table.get_column(&column_name) {
                Some(col) => col,
                None => return Err(QueryError::ValidationError(
                    format!("The column {} doesn't exist in {}", column_name, table.name)))
            };
            if !value.has_type(&table_column.column_type) {
                return Err(
                    QueryError::ValidationError(
                        format!("Incorrect value type for column {}. Expected '{:?}' and got '{:?}'",
                                column_name, table_column.column_type, value))
                )
            };
        }
        Ok(())
    }

    fn run_insert(&self, query: asl::InsertQuery) -> Result<String, QueryError> {
        let table = self.get_table(&query.table)?;
        let evaluated_expressions = query.evaluate_expressions()?;
        print!("Evaluated expressions: {:?}", evaluated_expressions);
        self.validate_insert(&table, &query, &evaluated_expressions)?;
        let result = format!("Running Insert {:?}", query);
        self.db_filesystem.insert_record(table, &asl::Record { values: evaluated_expressions })?;
        Ok(result)
    }

    fn run_create_table(&mut self, query: asl::CreateTableQuery) -> Result<String, QueryError> {
        if self.get_table(&query.table).is_ok() {
            return Err(QueryError::Conflict(query.table))
        }
        let result = format!("Running Create Table {:?}", query);
        let table = asl::Table {name: query.table, columns: query.columns};
        self.db_filesystem.create_table_files(&table)?;
        self.db_definition.tables.insert(table.name.clone(), table);
        self.db_filesystem.store_definitions(&self.db_definition)?;
        Ok(result)
    }

    fn run_drop_table(&mut self, query: asl::DropTableQuery) -> Result<String, QueryError> {
        self.db_filesystem.delete_table_files(self.get_table(&query.table)?)?;
        self.db_definition.tables.remove(&query.table);
        Ok(format!("Running Drop Table {:?}", query))
    }

    /**
    Parse and run query
    */
    pub fn run_query(&mut self, query: &str) -> Result<String, QueryError> {
        let query = sql_grammar::QueryParser::new().parse(query)?;
        match query {
            asl::Query::Select(q) => self.run_select(q),
            asl::Query::Insert(q) => self.run_insert(q),
            asl::Query::CreateTable(q) => self.run_create_table(q),
            asl::Query::DropTable(q) => self.run_drop_table(q),
        }
    }
}