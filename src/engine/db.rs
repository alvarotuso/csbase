use std::fmt;
use std::fs;
use std::io::{Read, Write};

use bincode;
use serde::{Serialize, Deserialize};
use lalrpop_util;

use crate::config::config;
use crate::engine::asl;
use crate::sql_grammar;


type ParseError<'input> = lalrpop_util::ParseError<usize, sql_grammar::Token<'input>, &'static str>;

#[derive(Debug)]
pub enum QueryError  {
    ParseError(String),
    IOError(std::io::Error),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl std::convert::From<std::io::Error> for QueryError {
    fn from(error: std::io::Error) -> Self {QueryError::IOError(error)}
}

impl <'input> std::convert::From<ParseError<'input>> for QueryError {
    fn from(error: ParseError<'input>) -> Self {QueryError::ParseError(format!("{:?}", error))}
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DatabaseDefinition {
    tables: Vec<asl::Table>,
}

#[derive(Debug)]
pub struct Database {
    pub db_definition: DatabaseDefinition,
}

impl Database {
    pub fn new() -> Database {
        Database {db_definition: DatabaseDefinition { tables: vec![] }}
    }

    fn get_base_path(&self) -> String {
        shellexpand::tilde(&config::DB_PATH).to_string()
    }

    fn get_path(&self, path: &str) -> String {
        format!("{}/{}", self.get_base_path(), path)
    }

    /**
    * Read and deserialize the database definition file
    */
    pub fn load_definitions(&mut self) -> std::io::Result<()> {
        let file = fs::File::open(self.get_path(config::TABLE_DEFINITIONS_FILE));
        match file {
            Ok(mut f) => {
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                self.db_definition = bincode::deserialize(buffer.as_slice()).unwrap();
                print!("Loaded db definition: {:#?}", self.db_definition);
                Ok(())
            },
            Err(e) => {
                print!("Error while loading db definition: {:#?}", e);
                Ok(())
            }
        }
    }

    /**
    * Serialize and store the database definition
    */
    pub fn store_definitions(&self) -> std::io::Result<()> {
        let mut file = fs::File::create(self.get_path(config::TABLE_DEFINITIONS_FILE))?;
        file.write_all(bincode::serialize(&self.db_definition).unwrap().as_slice())?;
        Ok(())
    }

    /**
    * Ensure that the database path exists
    */
    pub fn ensure_base_path(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.get_base_path())?;
        Ok(())
    }

    pub fn run_select(&self, query: asl::SelectQuery) -> Result<String, QueryError> {
        Ok(format!("Running Select {:?}", query))
    }

    pub fn run_insert(&self, query: asl::InsertQuery) -> Result<String, QueryError> {
        Ok(format!("Running Insert {:?}", query))
    }

    pub fn run_create_table(&mut self, query: asl::CreateTableQuery) -> Result<String, QueryError> {
        let result = format!("Running Create Table {:?}", query);
        let table = asl::Table {name: query.table, columns: query.columns};
        self.db_definition.tables.push(table);
        self.store_definitions()?;
        Ok(result)
    }

    pub fn run_drop_table(&self, query: asl::DropTableQuery) -> Result<String, QueryError> {
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