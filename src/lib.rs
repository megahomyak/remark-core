pub enum SubstitutionResult {
    Success(String),
    InputNotMatched,
}

pub enum Rule<'a> {
    Regex {
        pattern: regex::Regex,
        replacement: String,
    },
    Builtin(&'a dyn Fn(&str) -> SubstitutionResult),
}

impl Rule<'_> {
    fn process(&self, input: &str) -> SubstitutionResult {
        match self {
            Self::Regex {
                pattern,
                replacement,
            } => {
                if pattern.is_match(input) {
                    SubstitutionResult::Success(pattern.replace(input, replacement).to_string())
                } else {
                    SubstitutionResult::InputNotMatched
                }
            }
            Self::Builtin(function) => function(input),
        }
    }
}

pub fn execute(mut program: String, rules: &[Rule]) -> String {
    loop {
        let mut at_least_one_matched = false;
        for rule in rules {
            match rule.process(&program) {
                SubstitutionResult::Success(replacement) => {
                    at_least_one_matched = true;
                    program = replacement;
                }
                SubstitutionResult::InputNotMatched => (),
            }
        }
        if !at_least_one_matched {
            break;
        }
    }
    program
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(3, 3);
    }
}
