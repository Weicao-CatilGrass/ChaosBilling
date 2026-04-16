use mingling::setup::GeneralRendererSetup;

use crate::__completion_gen::CompletionDispatcher;
use crate::ThisProgram;
use crate::cli::dispatchers::*;

pub async fn entry() {
    let mut program = ThisProgram::new();

    // Add Completion
    program.with_dispatcher(CompletionDispatcher);

    // Add General Renderer
    program.with_setup(GeneralRendererSetup);

    // Setup `cobill`
    program.with_setup(ChaosBillingSetup);

    // Execute
    program.exec().await;
}
