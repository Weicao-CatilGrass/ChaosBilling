#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Who {
    name: String,
}

impl std::ops::Deref for Who {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl std::ops::DerefMut for Who {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.name
    }
}

impl From<String> for Who {
    fn from(s: String) -> Self {
        Who { name: s }
    }
}

impl From<&str> for Who {
    fn from(s: &str) -> Self {
        Who {
            name: s.to_string(),
        }
    }
}

impl Into<String> for Who {
    fn into(self) -> String {
        self.name
    }
}

impl std::fmt::Display for Who {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
