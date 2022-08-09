// Takes the value of the docker-mdns.enabled label and turns it into an enum.
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

// Only "true" will enable docker-mdns, all other values are disable it.
impl From<Option<&String>> for State {
    fn from(s: Option<&String>) -> Self {
        match s.map(String::as_ref) {
            Some("true") => Self::Enabled,
            _            => Self::Disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        // We need to be passing Option<&String>, so make some Strings.
        let true_one = String::from("true");
        let true_two = String::from("True");
        let false_one = String::from("false");

        let tests = vec![
            (Some(&true_one), State::Enabled),
            (Some(&true_two), State::Disabled),
            (Some(&false_one), State::Disabled),
            (None, State::Disabled),
        ];

        for test in tests  {
            let input = test.0;
            let expected = test.1;

            let ret = State::from(input);

            assert_eq!(ret, expected);
        }
    }
}
