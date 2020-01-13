
#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub column_type: Type,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>
}

#[derive(Debug)]
pub struct SelectQuery {
    pub table: String,
    pub columns: Vec<String>,
}

#[derive(Debug)]
pub struct InsertQuery {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
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

#[derive(Debug)]
pub enum Type {
    Str,
    Bool,
    Int,
    Float,
}

#[derive(Debug)]
pub enum Value {
    Str(String),
    Bool(bool),
    Int(i32),
    Float(f32),
}