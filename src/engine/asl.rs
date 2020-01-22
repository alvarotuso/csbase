use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::engine::errors::QueryError;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Column {
    pub name: String,
    pub column_type: Type,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ColumnValue {
    pub column: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub enum Comparator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, Clone)]
pub enum LogicOperator {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Value),
    Identifier(String),
    Op(Box<Expression>, Operator, Box<Expression>),
    Comp(Box<Expression>, Comparator, Box<Expression>),
    LogicOp(Box<Expression>, LogicOperator, Box<Expression>)
}

impl Expression {
    /**
    * Evaluates this expression. All Identifier variants must be turned into values first
    */
    pub fn evaluate(&self, identifier_values: Option<&HashMap<String, Value>>) -> Result<Value, QueryError> {
        match self {
            Expression::Value(value) => Ok(value.clone()),
            Expression::Identifier(name) => match identifier_values {
                Some(identifier_values) => match identifier_values.get(name) {
                    Some(value) => Ok(value.clone()),
                    None => Err(QueryError::ValidationError(
                        format!("Identifier {} not present in provided values", name)))
                },
                None => Err(QueryError::ValidationError(String::from(
                    "Identifier values not provided but they were used in this expression")))
            },
            Expression::Op(exp1, operator, exp2) => {
                let value1 = exp1.evaluate(identifier_values)?;
                let value2 = exp2.evaluate(identifier_values)?;
                match operator {
                    Operator::Add => value1 + value2,
                    Operator::Subtract => value1 - value2,
                    Operator::Multiply => value1 * value2,
                    Operator::Divide => value1 / value2,
                }
            },
            Expression::Comp(exp1, comparator, exp2) => {
                let value1 = exp1.evaluate(identifier_values)?;
                let value2 = exp2.evaluate(identifier_values)?;
                Ok(Value::Bool(
                    match comparator {
                        Comparator::Eq => value1 == value2,
                        Comparator::Neq => value1 != value2,
                        Comparator::Gt => value1 > value2,
                        Comparator::Gte => value1 >= value2,
                        Comparator::Lt => value1 < value2,
                        Comparator::Lte => value1 <= value2,
                    }
                ))
            },
            Expression::LogicOp(exp1, logic_operator, exp2) => {
                let value1 = exp1.evaluate(identifier_values)?.get_bool()?;
                let value2 = exp2.evaluate(identifier_values)?.get_bool()?;
                Ok(Value::Bool(
                    match logic_operator {
                        LogicOperator::And => value1 && value2,
                        LogicOperator::Or => value1 || value2,
                    }
                ))
            }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Type {
    Str,
    Bool,
    Int,
    Float,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    Str(String),
    Bool(bool),
    Int(i32),
    Float(f32),
}

impl std::ops::Div for Value {
    type Output = Result<Self, QueryError>;

    fn div(self, other: Self) -> Self::Output {
        match self {
            Value::Int(value1) => match other {
                Value::Int(value2) =>
                    if value2 != 0 {
                        Ok(Value::Float((value1 as f32) / (value2 as f32)))
                    } else {
                        Err(QueryError::ValidationError(String::from("Division by 0")))
                    },
                Value::Float(value2) =>
                    if value2 != 0.0 {
                        Ok(Value::Float((value1 as f32) / value2))
                    } else {
                        Err(QueryError::ValidationError(String::from("Division by 0")))
                    },
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            Value::Float(value1) => match other {
                Value::Int(value2) => Ok(Value::Float(value1 / (value2 as f32))),
                Value::Float(value2) => Ok(Value::Float(value1 / value2)),
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Result<Self, QueryError>;

    fn mul(self, other: Self) -> Self::Output {
        match self {
            Value::Int(value1) => match other {
                Value::Int(value2) => Ok(Value::Int(value1 * value2)),
                Value::Float(value2) => Ok(Value::Float((value1 as f32) * value2)),
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            Value::Float(value1) => match other {
                Value::Int(value2) => Ok(Value::Float(value1 * (value2 as f32))),
                Value::Float(value2) => Ok(Value::Float(value1 * value2)),
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
        }
    }
}

impl std::ops::Add for Value {
    type Output = Result<Self, QueryError>;

    fn add(self, other: Self) -> Self::Output {
        match self {
            Value::Int(value1) => match other {
                Value::Int(value2) => Ok(Value::Int(value1 + value2)),
                Value::Float(value2) => Ok(Value::Float((value1 as f32) + value2)),
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            Value::Float(value1) => match other {
                Value::Int(value2) => Ok(Value::Float(value1 + (value2 as f32))),
                Value::Float(value2) => Ok(Value::Float(value1 + value2)),
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            Value::Str(value1) => match other {
                Value::Str(value2) => Ok(Value::Str(value1 + &value2)),
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Result<Self, QueryError>;

    fn sub(self, other: Self) -> Self::Output {
        match self {
            Value::Int(value1) => match other {
                Value::Int(value2) => Ok(Value::Int(value1- value2)),
                Value::Float(value2) => Ok(Value::Float((value1 as f32) - value2)),
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            Value::Float(value1) => match other {
                Value::Int(value2) => Ok(Value::Float(value1 - (value2 as f32))),
                Value::Float(value2) => Ok(Value::Float(value1 - value2)),
                _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
            },
            _ => Err(QueryError::ValidationError(String::from("Invalid types for operator"))),
        }
    }
}

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Int(value1) => match other {
                Value::Int(value2) => value1 == value2,
                Value::Float(value2) => (*value1 as f32) == *value2,
                _ => false,
            },
            Value::Float(value1) => match other {
                Value::Int(value2) => *value1 == (*value2 as f32),
                Value::Float(value2) => *value1  == *value2,
                _ => false,
            },
            Value::Str(value1) => match other {
                Value::Str(value2) => value1 == value2,
                _ => false,
            },
            Value::Bool(value1) => match other {
                Value::Bool(value2) => value1 == value2,
                _ => false,
            },
        }
    }
}

impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Value::Int(value1) => match other {
                Value::Int(value2) => Some(value1.cmp(value2)),
                Value::Float(value2) => Some(value1.cmp(&(*value2 as i32))),
                _ => None,
            },
            Value::Float(value1) => match other {
                Value::Int(value2) => value1.partial_cmp(&(*value2 as f32)),
                Value::Float(value2) => value1.partial_cmp(&value2),
                _ => None,
            },
            Value::Str(value1) => match other {
                Value::Str(value2) => Some(value1.cmp(value2)),
                _ => None,
            },
            Value::Bool(_) => None,
        }
    }
}

impl Value {
    /**
    * Build value from bytes
    */
    pub fn from_be_bytes(bytes: Vec<u8>, value_type: &Type) -> Value {
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
    * Get the type of this Value
    */
    pub fn get_type(&self) -> Type {
        match self {
            Value::Str(_) => Type::Str,
            Value::Bool(_) => Type::Bool,
            Value::Int(_) => Type::Int,
            Value::Float(_) => Type::Float,
        }
    }

    /**
    * Test if a value has the provided type
    */
    pub fn has_type(&self, value_type: &Type) -> bool {
        if let
            (Type::Str, Type::Str) |
            (Type::Bool, Type::Bool) |
            (Type::Int, Type::Int) |
            (Type::Float, Type::Float)
        = (self.get_type(), value_type) {
            true
        } else {
            false
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

    pub fn get_bool(&self) -> Result<bool, QueryError> {
        match self {
            Value::Bool(value) => Ok(value.clone()),
            _ => Err(QueryError::ValidationError(String::from("Value is not boolean")))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectQuery {
    pub table: String,
    pub columns: Vec<String>,
    pub condition: Option<Box<Expression>>,
}

#[derive(Debug, Clone)]
pub struct InsertQuery {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Box<Expression>>,
}

impl InsertQuery {
    pub fn evaluate_expressions(&self) -> Result<Vec<Value>, QueryError> {
        let mut evaluated_expressions = Vec::new();
        for expression in &self.values {
            evaluated_expressions.push((*expression).evaluate(Option::None)?);
        }
        Ok(evaluated_expressions)
    }
}

#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub table: String,
    pub column_values: Vec<ColumnValue>,
}

#[derive(Debug, Clone)]
pub struct DeleteQuery {
    pub table: String,
}

#[derive(Debug, Clone)]
pub struct CreateTableQuery {
    pub table: String,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone)]
pub struct DropTableQuery {
    pub table: String,
}

#[derive(Debug, Clone)]
pub enum Query {
    Select(SelectQuery),
    Insert(InsertQuery),
    CreateTable(CreateTableQuery),
    DropTable(DropTableQuery),
}