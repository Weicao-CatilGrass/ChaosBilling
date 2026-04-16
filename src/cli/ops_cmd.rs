use std::{
    env::current_dir,
    fs::{self, create_dir_all},
    path::PathBuf,
};

use mingling::{
    AnyOutput,
    macros::{chain, dispatcher, pack, r_println, renderer},
    marker::NextProcess,
    parser::Picker,
};

use crate::{
    ThisProgram,
    cli::{consts::BILL_WORKSPACE_CONFIG_FILE, io_error::IOError},
};

dispatcher!("init", InitHereCommand => InitEntry);
dispatcher!("create", CreateCommand => CreateEntry);

pack!(StateCreateWorkspace = PathBuf);

#[chain]
pub async fn handle_init_command(_prev: InitEntry) -> NextProcess {
    let current_dir = match current_dir() {
        Ok(d) => d,
        Err(e) => return AnyOutput::new(IOError::from(e)).route_renderer(),
    };
    StateCreateWorkspace::new(current_dir).to_chain()
}

#[chain]
pub async fn handle_create_command(prev: CreateEntry) -> NextProcess {
    let path = pick_path(prev.inner);
    StateCreateWorkspace::new(path).to_chain()
}

#[chain]
pub async fn handle_state_create_workspace(prev: StateCreateWorkspace) -> NextProcess {
    let dir = prev.inner;
    let file = dir.join(BILL_WORKSPACE_CONFIG_FILE);

    match create_dir_all(&dir) {
        Ok(d) => d,
        Err(e) => return AnyOutput::new(IOError::from(e)).route_renderer(),
    };

    if file.exists() {
        return AnyOutput::new(WorkspaceConfigAlreadyExists::new(dir)).route_renderer();
    }

    if let Err(e) = fs::write(file, "") {
        return AnyOutput::new(IOError::from(e)).route_renderer();
    }

    StateWorkspaceCreated::new(dir).to_render()
}

pack!(StateWorkspaceCreated = PathBuf);

#[renderer]
pub fn render_workspace_created(prev: StateWorkspaceCreated) {
    r_println!("Workspace created at: {:?}", prev.inner);
}

pack!(WorkspaceConfigAlreadyExists = PathBuf);

#[renderer]
pub fn render_workspace_config_already_exists(prev: WorkspaceConfigAlreadyExists) {
    r_println!("Workspace config already exists: {:?}", prev.inner);
}

fn pick_path(args: Vec<String>) -> PathBuf {
    let path = Picker::<()>::new(args)
        .pick::<String>(())
        .unpack_directly()
        .0;
    PathBuf::from(path)
}
