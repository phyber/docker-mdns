// Docker mDNS
//
// Turn event actions into a nice enum
#![forbid(unsafe_code)]

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Action {
    Die,
    Other,
    Start,
}

impl From<&Option<String>> for Action {
    fn from(f: &Option<String>) -> Self {
        match f.as_ref().map(String::as_ref) {
            Some("die")   => Self::Die,
            Some("start") => Self::Start,
            _             => Self::Other,
        }
    }
}
