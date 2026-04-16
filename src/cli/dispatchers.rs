use mingling::{Program, macros::program_setup};

use crate::ThisProgram;
use crate::cli::ops_cmd::{CreateCommand, InitHereCommand};

#[program_setup]
pub fn chaos_billing_setup(program: &mut Program<ThisProgram, ThisProgram>) {
    program.with_dispatcher(InitHereCommand);
    program.with_dispatcher(CreateCommand);

    // program.with_dispatcher(CalculateCommand);
}
