#[macro_use] extern crate lalrpop_util;

extern crate bit_vec;
extern crate lalrpop;

mod config;
mod engine;

use std::io;

lalrpop_mod!(pub sql_grammar, "/grammar/sql_grammar.rs"); // synthesized by LALRPOP


fn main() {
    let mut database = engine::db::Database::new();
    database.bootstrap().expect("Error while starting the database");
    loop {
        print!("SQL> ");
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("Failed to read command");
        let result = match database.run_query(&command) {
            Ok(result) => result,
            Err(e) => format!("{:?}", e)
        };
        println!("{:?}", result);
    }
}
