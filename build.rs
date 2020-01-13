extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .force_build(true)
        .process_current_dir()
        .unwrap();
}