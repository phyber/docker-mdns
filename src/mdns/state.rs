// Docker mDNS
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Disabled,
    Enabled,
}

impl Default for State {
    fn default() -> Self {
        Self::Disabled
    }
}

impl From<Option<&String>> for State {
    fn from(s: Option<&String>) -> Self {
        match s.map(String::as_ref) {
            Some("true") => Self::Enabled,
            _            => Self::Disabled,
        }
    }
}
