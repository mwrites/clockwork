mod cli;
mod config;
mod deps;
mod errors;
mod parser;
mod processor;

use {
    cli::app,
    errors::CliError,
    processor::process,
};

fn main() -> Result<(), CliError> {
    process(&app().get_matches()).map_err(|e| {
        eprintln!("{}", e);
        e
    })
}
