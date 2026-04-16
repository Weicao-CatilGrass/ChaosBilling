#[macro_export]
macro_rules! string_vec {
    ($($elem:expr),* $(,)?) => {
        vec![$($elem.to_string()),*]
    };
}
