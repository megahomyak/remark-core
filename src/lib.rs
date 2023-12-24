use std::collections::HashMap;

// --------------------------------- BUT THIS IS GOLD VVVVVVVVVVVVVV -------------------

pub struct ProgramSequence<Part> {
    pub first_part: Program<Part>,
    pub rest: Vec<Program<Part>>,
}

pub enum ProgramPart {
    ProgramSequence(ProgramSequence<ProgramPart>),
    String(String),
}

pub enum ReplacementPart {
    ProgramSequence(ProgramSequence<ReplacementPart>),
    String(String),
    ReplacementMarker(String),
}

pub struct Program<Part> {
    pub parts: Vec<Part>,
}

pub enum ContextAction {
    AddSubstitution { name: String, content: Substitution },
}

pub struct FunctionResult {
    pub substitution: ProgramSequence<ProgramPart>,
    pub context_actions: Vec<ContextAction>,
}

pub struct Parameters<'a> {
    values: Vec<&'a str>,
}

impl<'a> Parameters<'a> {
    fn get(&self, index: usize) -> &str {
        self.values.get(index).unwrap_or(&"")
    }
}

pub enum Substitution {
    String { replacement: Program<ReplacementPart> },
    Function(Box<dyn FnMut(&Parameters) -> FunctionResult>),
}

pub struct Executor {
    pub substitutions: HashMap<String, Substitution>,
}

impl Program<ProgramPart> {
    fn execute(&self, executor: &mut Executor) -> String {
        let mut result = String::new();
        for part in self.parts {
            match part {
                ProgramPart::String(string) => result.push_str(&string),
                ProgramPart::ProgramSequence(program_sequence) => {
                    let name = program_sequence.first_part.execute(executor);
                    let parameters = Parameters { values:  }
                    if let Some(substitution) = executor.substitutions.get(name) {
                        match substitution {
                            Substitution::Function(function) => function()
                        }
                    } else {
                        result.push_str(substitution)
                    }
                }
            }
        }
    }
}

// --------------------------------- THIS IS OLD VVVVVVVVVVVVVVVVVVV -------------------
impl Executor {
    pub fn execute(&mut self, mut program: String) -> String {
        'cycle: loop {
            let mut chars = program.char_indices();
            program = 'substitution: loop {
                let mut opening_index = None;
                let mut parts = Vec::new();
                let mut last_string_part = String::new();
                while let Some((current_index, c)) = chars.next() {
                    match c {
                        '(' => {
                            parts.clear();
                            last_string_part.clear();
                            opening_index = Some(current_index)
                        }
                        '\\' => match chars.next() {
                            Some((_index, escaped_c)) => {
                                if !matches!(escaped_c, '\\' | '(' | ';' | ')') {
                                    last_string_part.push(c);
                                }
                                last_string_part.push(escaped_c);
                            }
                            None => break 'cycle,
                        },
                        ';' => {
                            last_string_part.shrink_to_fit();
                            parts.push(last_string_part);
                            last_string_part = String::new();
                        }
                        ')' => {
                            last_string_part.shrink_to_fit();
                            parts.push(last_string_part);
                            if let Some(opening_index) = opening_index {
                                let mut parts_iter = parts.iter().map(|part| &part[..]);
                                let name = parts_iter.next().unwrap_or("");
                                let Some(substitution) = self.substitutions.get_mut(name) else {
                                    continue 'substitution;
                                };
                                let parameters = Parameters {
                                    values: parts_iter.collect(),
                                };
                                let substitution = match substitution {
                                    Substitution::String { replacement } => {
                                        let mut chars = replacement.char_indices();
                                        let mut substitution = String::new();
                                        let mut opening_index = None;
                                        let mut key_name = String::new();
                                        while let Some((current_index, c)) = chars.next() {
                                            match c {
                                                '$' => match opening_index {
                                                    None => opening_index = Some(current_index),
                                                    Some(_) => {
                                                        opening_index = None;
                                                        if let Ok(index) = key_name.parse() {
                                                            substitution
                                                                .push_str(parameters.get(index));
                                                        };
                                                    }
                                                },
                                                '\\' => match chars.next() {
                                                    Some((_current_index, escaped_c)) => {
                                                        if let Some(_) = opening_index {
                                                            if !matches!(escaped_c, '$' | '\\') {
                                                                key_name.push(c);
                                                            }
                                                            key_name.push(escaped_c);
                                                        }
                                                    }
                                                    None => substitution.push(c),
                                                },
                                                _ => substitution.push(c),
                                            }
                                        }
                                        substitution
                                    }
                                    Substitution::Function(function) => {
                                        let result = function(&parameters);
                                        for action in result.context_actions {
                                            match action {
                                                ContextAction::AddSubstitution {
                                                    name,
                                                    content,
                                                } => {
                                                    self.substitutions.insert(name, content);
                                                }
                                            }
                                        }
                                        result.substitution
                                    }
                                };
                                break 'substitution format!(
                                    "{}{}{}",
                                    &program[..opening_index],
                                    substitution,
                                    chars.as_str(),
                                );
                            }
                        }
                        c => {
                            if let Some(_) = opening_index {
                                last_string_part.push(c)
                            }
                        }
                    }
                }
                break 'cycle;
            }
        }
        program
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
        };
        assert_eq!(execute(&mut context, "Hello!".into()), "Hi!");
    }

    #[test]
    fn countdown() {
        let mut context = ExecutionContext {
            rules: [
                Rule::Builtin {
                    pattern: Regex::new(r"\(minus (\d+) (\d+)\)").unwrap(),
                    replacer: Box::new(|captures: &regex::Captures| {
                        let minuend: usize = captures.get(1).unwrap().as_str().parse().unwrap();
                        let subtrahend: usize = captures.get(2).unwrap().as_str().parse().unwrap();
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
                    pattern: Regex::new(r"\(countdown 1\)").unwrap(),
                    replacement: "1".into(),
                },
            ]
            .into(),
        };
        assert_eq!(execute(&mut context, "(countdown 5)".into()), "5 4 3 2 1");
    }

    #[test]
    fn no_matches() {
        let mut context = ExecutionContext {
            rules: [Rule::Regex {
                pattern: Regex::new(r"abc").unwrap(),
                replacement: "def".into(),
            }]
            .into(),
        };
        assert_eq!(
            execute(&mut context, "Hello, world!".into()),
            "Hello, world!"
        );
    }
}
