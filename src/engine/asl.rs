use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Column {
    pub name: String,
    pub column_type: Type,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>
}

impl Table {
    /**
    * Get a reference to the column that matches column_name
    */
    pub fn get_column(&self, column_name: &str) -> Option<&Column> {
        self.columns.iter().find(| column | &column.name == column_name)
    }
}

#[derive(Debug)]
pub struct ColumnValue {
    pub column: String,
    pub value: Value,
}

#[derive(Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum Comparator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug)]
pub enum LogicOperator {
    And,
    Or,
}

#[derive(Debug)]
pub enum Expression {
    Value(Value),
    Identifier(String),
    Op(Box<Expression>, Operator, Box<Expression>),
}

#[derive(Debug)]
pub enum LogicExpression {
    Comparison(Box<Expression>, Comparator, Box<Expression>),
    LogicExpression(Box<LogicExpression>, LogicOperator, Box<LogicExpression>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Type {
    Str,
    Bool,
    Int,
    Float,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Value {
    Str(String),
    Bool(bool),
    Int(i32),
    Float(f32),
}

impl Value {
    /**
    * Build value from bytes
    */
    pub fn from_be_bytes(value_type: &Type, bytes: Vec<u8>) -> Value {
        match value_type {
            Type::Str => Value::Str(String::from_utf8(bytes).unwrap()),
            Type::Bool => Value::Bool(if bytes[0] == 1u8 { true } else { false }),
            Type::Int => Value::Int(i32::from_be_bytes(
                [bytes[0], bytes[1], bytes[2], bytes[3]])),
            Type::Float => Value::Float(f32::from_be_bytes(
                [bytes[0], bytes[1], bytes[2], bytes[3]])),
        }
    }

    /**
    * Test if a value has the provided type
    */
    pub fn has_type(&self, value_type: &Type) -> bool {
        match value_type {
            Type::Str => match &self {
                Value::Str(_) => true,
                _ => false,
            },
            Type::Bool => match &self {
                Value::Bool(_) => true,
                _ => false,
            },
            Type::Int => match &self {
                Value::Int(_) => true,
                _ => false,
            },
            Type::Float => match &self {
                Value::Float(_) => true,
                _ => false,
            },
        }
    }

    /**
    * Get u8 array of the value
    */
    pub fn to_be_bytes(self) -> Vec<u8> {
        match self {
            Value::Str(val) => val.into_bytes(),
            Value::Int(val) => val.to_be_bytes().to_vec(),
            Value::Float(val) => val.to_be_bytes().to_vec(),
            Value::Bool(val) => (if val { 1u8 } else { 0u8 }).to_be_bytes().to_vec(),
        }
    }
}

#[derive(Debug)]
pub struct SelectQuery {
    pub table: String,
    pub columns: Vec<String>,
    pub condition: Option<Box<LogicExpression>>,
}

#[derive(Debug)]
pub struct InsertQuery {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}

#[derive(Debug)]
pub struct UpdateQuery {
    pub table: String,
    pub column_values: Vec<ColumnValue>,
}

#[derive(Debug)]
pub struct DeleteQuery {
    pub table: String,
}

#[derive(Debug)]
pub struct CreateTableQuery {
    pub table: String,
    pub columns: Vec<Column>,
}

#[derive(Debug)]
pub struct DropTableQuery {
    pub table: String,
}

#[derive(Debug)]
pub enum Query {
    Select(SelectQuery),
    Insert(InsertQuery),
    CreateTable(CreateTableQuery),
    DropTable(DropTableQuery),
}