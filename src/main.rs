extern crate core;

use std::error::Error;
use crate::preprocessor::Preprocessor;
use crate::variables::VariableCollection;
use clap::Parser;

mod preprocessor;
mod block;
mod command;
mod variables;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    context: Option<std::path::PathBuf>,

    #[arg(short = 'v', long = "variable")]
    variables: Vec<String>,

    input_file: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let context: String;
    if cli.context.is_some() {
        context = String::from(cli.context.unwrap().canonicalize().unwrap().to_str().unwrap());
    } else {
        context = Preprocessor::get_absolute_path_context(cli.input_file.to_str().unwrap());
    }

    let mut var_collection = VariableCollection::new();

    for variable in cli.variables {
        let split: Vec<&str> = variable.split("=").collect();
        var_collection = var_collection.set(split[0], split[1]);
    }

    print!("{}", Preprocessor::process_file(
        cli.input_file.to_str().unwrap(),
        var_collection,
        &context
    ).format());

    Ok(())
}
