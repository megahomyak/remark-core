use std::{collections::HashMap, fmt::Display};

enum Token {
    GroupOpener,
    GroupCloser,
    ParameterSeparator,
    PlainCharacter(char),
    EscapedCharacter(char),
}

impl Token {
    fn parse(c: char, it: &mut impl Iterator<Item = char>) -> Self {
        match c {
            '(' => Token::GroupOpener,
            ')' => Token::GroupCloser,
            ';' => Token::ParameterSeparator,
            '\\' => match it.next() {
                None => Token::PlainCharacter(c),
                Some(c) => Token::EscapedCharacter(c),
            },
            c => Token::PlainCharacter(c),
        }
    }

    fn repr(&self, s: &mut String) {
        let c = match self {
            Self::GroupOpener => '(',
            Self::GroupCloser => ')',
            Self::ParameterSeparator => ';',
            Self::EscapedCharacter(c) => {
                s.push('\\');
                *c
            }
            Self::PlainCharacter(c) => *c,
        };
        s.push(c);
    }
}

fn tokenize(s: &str) -> Vec<Token> {
    let mut chars = s.chars();
    let mut tokens = Vec::new();
    while let Some(c) = chars.next() {
        tokens.push(Token::parse(c, &mut chars));
    }
    tokens
}

pub struct Parameters {
    values: Vec<String>,
}

impl Parameters {
    pub fn get(&self, index: usize) -> &str {
        self.values.get(index).map(|s| &s[..]).unwrap_or("")
    }
}

pub trait Substitution {
    fn execute(&mut self, parameters: &Parameters) -> SubstitutionResult;
}

impl<T: Fn(&Parameters) -> SubstitutionResult> Substitution for T {
    fn execute(&mut self, parameters: &Parameters) -> SubstitutionResult {
        self(parameters)
    }
}

pub enum SubstitutionAction {
    NewSubstitution(Box<dyn Substitution>),
}

pub struct SubstitutionResult {
    pub replacement: String,
    pub actions: Vec<SubstitutionAction>,
}

pub struct ExecutionContext {
    substitutions: HashMap<String, Box<dyn Substitution>>,
}

struct Group {
    name: String,
    parameters: Vec<String>,
}

struct FoundGroup<'a> {
    before: &'a str,
    group: Group,
    after: &'a str,
}

impl ExecutionContext {
    fn find_group(program: &str) -> Option<FoundGroup> {
        let mut chars = program.char_indices();
        let mut opening_index = None;
        let mut group_items = Vec::new();
        let mut last_group_item = String::new();
        while let Some((index, c)) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    None => return None,
                    Some((_index, c)) => last_group_item.push(c),
                }
            } else if c == '(' {
                opening_index = Some(index);
                group_items = Vec::new();
                last_group_item = String::new();
            } else if let Some(opening_index) = opening_index {
                if c == ';' {
                    group_items.push(last_group_item);
                    last_group_item = String::new();
                } else if c == ')' {
                    group_items.push(last_group_item);
                    let mut group_items = group_items.into_iter();
                    let name = group_items.next().unwrap();
                    let parameters: Vec<String> = group_items.collect();
                    return Some(FoundGroup {
                        before: &program[..opening_index],
                        group: Group {
                            parameters,
                            name,
                        },
                        after: chars.as_str(),
                    });
                } else {
                    last_group_item.push(c);
                }
            }
        }
        None
    }

    pub fn execute(program: String) -> String {
        let mut tokens = tokenize(&program);
        loop {

        }
        let mut result = String::new();
        for token in tokens {
            token.repr(&mut result);
        }
        result
    }
}
