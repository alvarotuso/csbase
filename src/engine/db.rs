use std::fmt;

use crate::engine::asl;

#[derive(Debug)]
pub enum QueryErrorType {
    SyntaxError,
}

#[derive(Debug)]
pub struct QueryError {
    error_type: QueryErrorType,
    message: String,
}

impl QueryError {
    fn new(error_type: QueryErrorType, message: String) -> QueryError {
        QueryError {error_type, message}
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

/**
Parse query into ASL
*/
pub fn parse_query(query: &str) -> Result<asl::Query, QueryError> {
    match sql_grammar::QueryParser::new().parse(query) {
        Ok(q) => Ok(q),
        Err(e) => Err(QueryError::new(QueryErrorType::SyntaxError,format!("{}", e)))
    }
}

#[derive(Debug)]
pub struct Database {
    tables: Vec<asl::Table>,
}

impl Database {
    pub fn new() -> Database {
        Database {tables: vec![]}
    }

    /**
    Run a select query
    */
    pub fn run_select(query: &asl::SelectQuery) -> Result<&str, QueryError> {
        Ok(&format!("Running Select {:?}", query))
    }

    /**
    Run an insert query
    */
    pub fn run_insert(query: &asl::InsertQuery) -> Result<&str, QueryError> {
        Ok(&format!("Running Insert {:?}", query))
    }

    /**
    Run an insert query
    */
    pub fn run_create_table(query: &asl::CreateTableQuery) -> Result<&str, QueryError> {
        Ok(&format!("Running Create Table {:?}", query))
    }

    /**
    Run an insert query
    */
    pub fn run_drop_table(query: &asl::DropTableQuery) -> Result<&str, QueryError> {
        Ok(&format!("Running Drop Table {:?}", query))
    }

    /**
    Parse and run query
    */
    pub fn run_query(&self, query: &str) -> Result<&str, QueryError> {
        let query = match parse_query(query) {
            Ok(query) => query,
            Err(e) =>  return Err(e)
        };
        match query {
            asl::Query::Select(q) => self.run_select(&q),
            asl::Query::Insert(q) => self.run_insert(&q),
            asl::Query::CreateTable(q) => self.run_create_table(&q),
            asl::Query::DropTable(q) => self.run_drop_table(&q),
        }
    }
}