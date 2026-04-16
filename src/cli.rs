use std::{fs::create_dir_all, io::ErrorKind, path::PathBuf};

use mingling::{
    AnyOutput, Groupped,
    macros::{chain, dispatcher, gen_program, pack, r_println, renderer},
    marker::NextProcess,
    parser::Picker,
    setup::GeneralRendererSetup,
};
use serde::Serialize;

use crate::{
    bill::{BillItem, Bills, SplitResult},
    calc::calculate_from,
    display::SimpleTable,
    edit::{get_default_editor, input_with_editor_cutsom},
    error::BillSplitError,
    string_vec,
};

pub async fn entry() {
    let mut program = ThisProgram::new();

    // Add Completion
    program.with_dispatcher(CompletionDispatcher);

    // Add General Renderer
    program.with_setup(GeneralRendererSetup);

    // Add Dispatchers
    program.with_dispatchers((
        ClearAllBillCommand,
        AddBillCommand,
        EditCommand,
        ListAllBillCommand,
    ));

    program.with_dispatchers((
        EditWithViCommand,
        EditWithVimCommand,
        EditWithNvimCommand,
        EditWithHelixCommand,
        EditWithNanoCommand,
    ));

    // Execute
    program.exec().await;
}

dispatcher!("clear", ClearAllBillCommand => ClearAllBillEntry);
dispatcher!("add", AddBillCommand => AddBillEntry);
dispatcher!("edit", EditCommand => EditEntry);
dispatcher!("ls", ListAllBillCommand => ListAllBillEntry);

#[chain]
async fn do_clear_cmd(_prev: ClearAllBillEntry) -> NextProcess {
    op_bills(|b| b.clear_items());
    Empty::new(()).to_render()
}

pack!(StateAddBillItem = BillItem);

#[chain]
async fn parse_add_cmd(prev: AddBillEntry) -> NextProcess {
    let picked = Picker::new(prev.inner)
        .pick_or_route::<f64>(["--paid", "-p"], PaidRequired::new(()).to_render())
        .pick_or_route::<Vec<String>>(["--for", "-f"], ForMembersRequired::new(()).to_render())
        .pick_or::<String>(
            ["--reason", "-r", "--message", "-m"],
            "No reason".to_string(),
        )
        .pick_or_route::<String>((), MemberRequired::new(()).to_render())
        .unpack();

    match picked {
        Ok((paid, for_members, reason, who)) => {
            let bill_item = BillItem {
                who_paid: who.into(),
                reason,
                paid,
                split: for_members.iter().map(|i| i.as_str().into()).collect(),
            };
            let state = StateAddBillItem::new(bill_item);
            AnyOutput::new(state).route_chain()
        }
        Err(e) => e,
    }
}

#[chain]
async fn handle_add_bill_item(prev: StateAddBillItem) -> NextProcess {
    op_bills(|b| {
        b.add_item(prev.inner);
    });
    Empty::new(()).to_render()
}

#[derive(Serialize, Groupped)]
struct StateListBills {
    optimize: bool,
}

pack!(ResultBills = Bills);
pack!(ResultSplitResult = SplitResult);
pack!(ErrorDuplicateSplitMembers = ());
pack!(ErrorNegativePaidAmount = ());

#[chain]
async fn parse_ls_cmd(prev: ListAllBillEntry) -> NextProcess {
    let optimize = Picker::<()>::new(prev.inner)
        .pick::<bool>(["-O", "--optimize"])
        .unpack_directly()
        .0;
    let state = StateListBills { optimize };
    AnyOutput::new(state).route_chain()
}

#[chain]
async fn handle_list_bills(prev: StateListBills) -> NextProcess {
    if prev.optimize {
        let bills = read_bills();
        match calculate_from(bills) {
            Ok(r) => AnyOutput::new(ResultSplitResult::new(r)).route_renderer(),
            Err(BillSplitError::DuplicateSplitMembers) => {
                AnyOutput::new(ErrorDuplicateSplitMembers::new(())).route_renderer()
            }
            Err(BillSplitError::NegativePaidAmount) => {
                AnyOutput::new(ErrorNegativePaidAmount::new(())).route_renderer()
            }
        }
    } else {
        let bills = read_bills();
        AnyOutput::new(ResultBills::new(bills)).route_renderer()
    }
}

#[renderer]
fn render_bills(prev: ResultBills) {
    r_println!("{}", prev.inner.table())
}

#[renderer]
fn render_split_result(prev: ResultSplitResult) {
    let mut table = SimpleTable::new(string_vec!["Who", "|", "Should Pay", "|", "To"]);
    for ((who, to), paid) in prev.inner.final_result {
        table.push_item(string_vec![who, "|", paid, "|", to]);
    }
    r_println!("{}", table)
}

pack!(StateEditBills = String); // Editor name
pack!(ErrorEditorNotFound = String);

#[chain]
async fn parse_edit_cmd(prev: EditEntry) -> NextProcess {
    let editor = Picker::<()>::new(prev.inner)
        .pick_or::<String>(["--editor", "-e"], get_default_editor())
        .unpack_directly()
        .0;
    let state = StateEditBills::new(editor);
    AnyOutput::new(state).route_chain()
}

#[chain]
async fn exec_edit_cmd(prev: StateEditBills) -> NextProcess {
    let text = match input_with_editor_cutsom(
        read_bills().table(),
        state_edit_file_path(),
        "#",
        prev.inner.clone(),
    ) {
        Ok(v) => v,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                return AnyOutput::new(ErrorEditorNotFound::new(prev.inner)).route_renderer();
            }
            _ => panic!("Error editing bills: {}", e),
        },
    };
    write_bills(Bills::from_table_str(text));
    Empty::new(()).to_render()
}

#[renderer]
fn render_error_editor_not_found(prev: ErrorEditorNotFound) {
    r_println!("Error: Editor \"{}\" not found", prev.inner);
}

#[renderer]
fn render_error_duplicate_split_members(_prev: ErrorDuplicateSplitMembers) {
    r_println!("Error: Duplicate members found in split list");
}

#[renderer]
fn render_error_negative_paid_amount(_prev: ErrorNegativePaidAmount) {
    r_println!("Error: Paid amount cannot be negative");
}

pack!(Empty = ());
pack!(PaidRequired = ());
pack!(ForMembersRequired = ());
pack!(MemberRequired = ());

#[renderer]
fn render_empty(_prev: Empty) {}

#[renderer]
fn render_paid_required(_prev: PaidRequired) {
    r_println!("Error: Paid amount required, use \"--paid\" or \"-p\"");
}

#[renderer]
fn render_for_members_required(_prev: ForMembersRequired) {
    r_println!("Error: For members required, use \"--for\" or \"-f\"");
}

#[renderer]
fn render_member_required(_prev: MemberRequired) {
    r_println!("Error: Member required");
}

fn cobill_dir() -> PathBuf {
    dirs::config_dir().unwrap().join(".cobill")
}

fn state_file_path() -> PathBuf {
    cobill_dir().join("state.yml")
}

fn state_edit_file_path() -> PathBuf {
    cobill_dir().join("edit.state.md")
}

fn read_bills() -> Bills {
    let dir = cobill_dir();
    create_dir_all(dir).unwrap();

    let state_file = state_file_path();
    if state_file.exists() {
        match std::fs::read_to_string(&state_file) {
            Ok(contents) => match serde_yaml::from_str(&contents) {
                Ok(bills) => bills,
                Err(_) => Bills::default(),
            },
            Err(_) => Bills::default(),
        }
    } else {
        Bills::default()
    }
}

fn write_bills(bills: Bills) {
    let state_file = state_file_path();
    let contents = serde_yaml::to_string(&bills).unwrap();
    std::fs::write(state_file, contents).unwrap();
}

fn op_bills<F: FnOnce(&mut Bills)>(op: F) {
    let mut bills = read_bills();
    op(&mut bills);
    write_bills(bills);
}

dispatcher!("vi", EditWithViCommand => EditWithViEntry);
dispatcher!("vim", EditWithVimCommand => EditWithVimEntry);
dispatcher!("nvim", EditWithNvimCommand => EditWithNvimEntry);
dispatcher!("helix", EditWithHelixCommand => EditWithHelixEntry);
dispatcher!("nano", EditWithNanoCommand => EditWithNanoEntry);

#[chain]
async fn edit_with_vi(_prev: EditWithViEntry) -> NextProcess {
    EditEntry::new(string_vec!["-e", "vi"]).to_chain()
}

#[chain]
async fn edit_with_vim(_prev: EditWithVimEntry) -> NextProcess {
    EditEntry::new(string_vec!["-e", "vim"]).to_chain()
}

#[chain]
async fn edit_with_nvim(_prev: EditWithNvimEntry) -> NextProcess {
    EditEntry::new(string_vec!["-e", "nvim"]).to_chain()
}

#[chain]
async fn edit_with_helix(_prev: EditWithHelixEntry) -> NextProcess {
    EditEntry::new(string_vec!["-e", "helix"]).to_chain()
}

#[chain]
async fn edit_with_nano(_prev: EditWithNanoEntry) -> NextProcess {
    EditEntry::new(string_vec!["-e", "nano"]).to_chain()
}

gen_program!();
