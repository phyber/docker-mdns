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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let tests = vec![
            (Some("die".to_string()), Action::Die),
            (Some("start".to_string()), Action::Start),
            (Some("foobar".to_string()), Action::Other),
            (None, Action::Other),
        ];

        for test in tests {
            let input = test.0;
            let expected = test.1;

            let action = Action::from(&input);

            assert_eq!(action, expected);
        }
    }
}
