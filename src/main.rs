use mingling::macros::gen_program;

mod bill;
mod calc;
mod cli;
mod error;
mod who;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() {
    cli::entry::entry().await
}

use crate::cli::calc_cmd::*;
use crate::cli::dispatchers::*;
use crate::cli::entry::*;
use crate::cli::io_error::*;
use crate::cli::ops_cmd::*;

gen_program!();
