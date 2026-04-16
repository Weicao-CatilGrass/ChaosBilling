use mingling::{
    Groupped,
    macros::{r_println, renderer},
};
use serde::Serialize;

use crate::ThisProgram;

#[derive(Groupped)]
pub struct IOError {
    inner: std::io::Error,
}

impl IOError {
    pub fn new(error: std::io::Error) -> Self {
        Self { inner: error }
    }
}

impl From<std::io::Error> for IOError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error)
    }
}

impl Serialize for IOError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("IOError", 2)?;
        state.serialize_field("kind", &self.inner.kind().to_string())?;
        state.serialize_field("info", &self.inner.to_string())?;
        state.end()
    }
}

#[renderer]
pub fn render_io_error(prev: IOError) {
    r_println!("{}: {}", prev.inner.kind(), prev.inner.to_string())
}
