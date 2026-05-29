/// How individual classes are wrapped.
#[derive(Debug, Default, Clone, Copy)]
pub enum ClassWrapping {
    #[default]
    NoWrapping,
    CommaSingleQuotes,
    CommaDoubleQuotes,
}

impl ClassWrapping {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClassWrapping::NoWrapping => "no-wrapping",
            ClassWrapping::CommaSingleQuotes => "comma-single-quotes",
            ClassWrapping::CommaDoubleQuotes => "comma-double-quotes",
        }
    }
}

impl<T: AsRef<str>> From<T> for ClassWrapping {
    fn from(s: T) -> Self {
        match s.as_ref() {
            "no-wrapping" => Self::NoWrapping,
            "comma-single-quotes" => Self::CommaSingleQuotes,
            "comma-double-quotes" => Self::CommaDoubleQuotes,
            _ => Self::NoWrapping,
        }
    }
}
