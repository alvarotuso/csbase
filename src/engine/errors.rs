use std::fmt;

use lalrpop_util;

use crate::sql_grammar;

type ParseError<'input> = lalrpop_util::ParseError<usize, sql_grammar::Token<'input>, &'static str>;

#[derive(Debug)]
pub enum QueryError  {
    ParseError(String),
    IOError(std::io::Error),
    NotFound(String),
    Conflict(String),
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

#[derive(Debug)]
pub enum SystemError {
    IOError(std::io::Error),
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl std::convert::From<std::io::Error> for SystemError {
    fn from(error: std::io::Error) -> Self {SystemError::IOError(error)}
}

