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
                            },
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
