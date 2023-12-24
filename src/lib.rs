use std::collections::HashMap;

fn split(s: &str) -> Option<(char, &str)> {
    let mut chars = s.chars();
    chars.next().map(|c| (c, chars.as_str()))
}

fn parse_char(s: &str) -> Option<(char, &str)> {
    split(s).and_then(|(c, s)| match c {
        ';' | '(' | ')' => None,
        '\\' => split(s),
    })
}

fn repeat<T, C: Extend<T>>(
    mut container: C,
    f: impl Fn(&str) -> Option<(T, &str)>,
    mut s: &str,
) -> (C, &str) {
    while let Some((item, rest)) = f(s) {
        container.extend(std::iter::once(item));
        s = rest;
    }
    (container, s)
}

fn parse_parameter(s: &str) -> Option<(String, &str)> {
    parse_char(s).map(|(c, s)| repeat(String::from(c), parse_char, s))
}

struct Group {
    name: String,
    rest: Vec<String>,
}

fn parse_group(s: &str) -> Result<(Group, &str), &str> {
    split(s).filter(|c| c == '(').and_then(|(_, s)|)
}

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

pub struct Parameters {
    values: Vec<String>,
}

impl Parameters {
    fn get(&self, index: usize) -> &str {
        self.values.get(index).map(|s| &s[..]).unwrap_or("")
    }
}

pub enum Substitution {
    String {
        replacement: Program<ReplacementPart>,
    },
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
                    let parameters = Parameters {
                        values: program_sequence
                            .rest
                            .into_iter()
                            .map(|part| part.execute(executor))
                            .collect(),
                    };
                    if let Some(substitution) = executor.substitutions.get(name) {
                        result.push_str(&match substitution {
                            Substitution::Function(function) => {
                                let result = function(&parameters);
                                result.substitution
                            }
                        })
                    } else {
                        result.push_str(substitution)
                    }
                }
            }
        }
        result
    }
}
