enum SubstitutionResult {
    Success {
        new_program: String,
        new_rule: Option<Rule>,
    },
    InputNotMatched,
}

pub struct ReplacementResult {
    pub new_rule: Option<Rule>,
    pub substitution: String,
}

pub trait Replacer {
    fn replace(&self, captures: &regex::Captures) -> ReplacementResult;
}

impl<T: Fn(&regex::Captures) -> ReplacementResult> Replacer for T {
    fn replace(&self, captures: &regex::Captures) -> ReplacementResult {
        self(captures)
    }
}

pub enum Rule {
    Regex {
        pattern: regex::Regex,
        replacement: String,
    },
    Builtin {
        pattern: regex::Regex,
        replacer: Box<dyn Replacer>,
    },
}

impl Rule {
    fn process(&self, input: &str) -> SubstitutionResult {
        match self {
            Self::Regex {
                pattern,
                replacement,
            } => {
                if pattern.is_match(input) {
                    SubstitutionResult::Success {
                        new_program: pattern.replace(input, replacement).to_string(),
                        new_rule: None,
                    }
                } else {
                    SubstitutionResult::InputNotMatched
                }
            }
            Self::Builtin { pattern, replacer } => match pattern.captures(input) {
                None => SubstitutionResult::InputNotMatched,
                Some(captures) => {
                    let execution_result = replacer.replace(&captures);
                    let new_program = function_string_builder::build(|mut collector| {
                        collector.collect(&input[..captures.get(0).unwrap().start()]);
                        collector.collect(&execution_result.substitution);
                        collector.collect(&input[captures.get(0).unwrap().end()..]);
                    });
                    SubstitutionResult::Success {
                        new_program,
                        new_rule: execution_result.new_rule,
                    }
                }
            },
        }
    }
}

pub struct ExecutionContext {
    pub program: String,
    pub rules: Vec<Rule>,
}

enum MatchingStatus {
    NoneMatched,
    Matched { new_rule: Option<Rule> },
}

pub fn execute(context: &mut ExecutionContext) {
    loop {
        let mut match_status = MatchingStatus::NoneMatched;
        for rule in context.rules.iter().rev() {
            match rule.process(&context.program) {
                SubstitutionResult::Success {
                    new_rule,
                    new_program,
                } => {
                    match_status = MatchingStatus::Matched { new_rule };
                    context.program = new_program;
                    break;
                }
                SubstitutionResult::InputNotMatched => (),
            }
        }
        match match_status {
            MatchingStatus::Matched { new_rule } => {
                if let Some(new_rule) = new_rule {
                    context.rules.push(new_rule)
                }
            }
            MatchingStatus::NoneMatched => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn hello() {
        let mut context = ExecutionContext {
            rules: [Rule::Regex {
                pattern: Regex::new(r"ello").unwrap(),
                replacement: "i".into(),
            }]
            .into(),
            program: "Hello!".into(),
        };
        execute(&mut context);
        assert_eq!(context.program, "Hi!");
    }

    #[test]
    fn countdown() {
        let mut context = ExecutionContext {
            rules: [
                Rule::Builtin {
                    pattern: Regex::new(r"\(minus (\d+) (\d+)\)").unwrap(),
                    replacer: Box::new(|captures: &regex::Captures| {
                        let minuend: u64 = captures.get(1).unwrap().as_str().parse().unwrap();
                        let subtrahend: u64 = captures.get(2).unwrap().as_str().parse().unwrap();
                        ReplacementResult {
                            new_rule: None,
                            substitution: format!("{}", minuend - subtrahend),
                        }
                    }),
                },
                Rule::Regex {
                    pattern: Regex::new(r"\(countdown (\d+)\)").unwrap(),
                    replacement: "$1 (countdown (minus $1 1))".into(),
                },
                Rule::Regex {
                    pattern: Regex::new(r"\(countdown 0\)").unwrap(),
                    replacement: "0".into(),
                },
            ]
            .into(),
            program: "(countdown 5)".into(),
        };
        execute(&mut context);
        assert_eq!(context.program, "5 4 3 2 1 0");
    }
}
