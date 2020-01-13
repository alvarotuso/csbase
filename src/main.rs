#[macro_use] extern crate lalrpop_util;

mod config;
mod engine;
mod fs;

use std::io;

use config::{Config, File, FileFormat};
lalrpop_mod!(pub sql_grammar, "/grammar/sql_grammar.rs"); // synthesized by LALRPOP


fn main() {
    fs::db::ensure_db_path().expect("Unable to verify or create db_path");
    loop {
        print!("SQL> ");
        let mut command = String::new();
        io::stdin().read_line(&mut command).expect("Failed to read command");
        println!("The command {} has been received", command.replace("\n", ""));
        let result = match engine::query::run_query(&command) {
            Ok(result) => result,
            Err(e) => format!("{:?}", e)
        };
        println!("{:?}", result);
    }
}
