#[derive(thiserror::Error, Debug)]
pub enum BillSplitError {
    #[error("Paid amount cannot be negative")]
    NegativePaidAmount,

    #[error("Duplicate split members found")]
    DuplicateSplitMembers,
}
