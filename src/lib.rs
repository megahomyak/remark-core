use std::collections::HashMap;

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

impl<T: FnMut(&Parameters) -> SubstitutionResult> Substitution for T {
    fn execute(&mut self, parameters: &Parameters) -> SubstitutionResult {
        self(parameters)
    }
}

pub enum SubstitutionAction {
    NewSubstitution {
        name: String,
        substitution: Box<dyn Substitution>,
    },
}

pub struct SubstitutionResult {
    pub replacement: String,
    pub actions: Vec<SubstitutionAction>,
}

pub struct Executor {
    pub substitutions: HashMap<String, Box<dyn Substitution>>,
}

impl Executor {
    fn find_group<'a>(
        &mut self,
        program: &'a str,
    ) -> Option<(&'a str, SubstitutionResult, &'a str)> {
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
            } else if let Some(unboxed_opening_index) = opening_index {
                if c == ';' {
                    group_items.push(last_group_item);
                    last_group_item = String::new();
                } else if c == ')' {
                    group_items.push(last_group_item);
                    let mut group_items_iter = group_items.into_iter();
                    let name = group_items_iter.next().unwrap();
                    if let Some(substitution) = self.substitutions.get_mut(&name) {
                        let parameters = Parameters {
                            values: group_items_iter.collect(),
                        };
                        let result = substitution.execute(&parameters);
                        return Some((&program[..unboxed_opening_index], result, chars.as_str()));
                    }
                    group_items = Vec::new();
                    last_group_item = String::new();
                    opening_index = None;
                } else {
                    last_group_item.push(c);
                }
            }
        }
        None
    }

    pub fn execute(&mut self, mut program: String) -> String {
        while let Some((before, substitution_result, after)) = self.find_group(&program) {
            program = format!("{}{}{}", before, substitution_result.replacement, after);
            for action in substitution_result.actions {
                match action {
                    SubstitutionAction::NewSubstitution { name, substitution } => {
                        self.substitutions.insert(name, substitution);
                    }
                }
            }
        }
        program
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_group<'a>(
        executor: &mut Executor,
        program: &'a str,
    ) -> Option<(&'a str, String, &'a str)> {
        executor
            .find_group(program)
            .map(|(before, group, after)| (before, group.replacement, after))
    }

    fn create_executor<const N: usize>(
        substitutions: [(&str, Box<dyn Substitution>); N],
    ) -> Executor {
        Executor {
            substitutions: HashMap::from(
                substitutions.map(|(name, substitution)| (name.to_owned(), substitution)),
            ),
        }
    }

    fn create_result(replacement: impl Into<String>) -> SubstitutionResult {
        SubstitutionResult {
            replacement: replacement.into(),
            actions: Vec::new(),
        }
    }

    #[test]
    fn test_group_finding() {
        let mut executor = create_executor([]);
        assert_eq!(find_group(&mut executor, "abc(def;ghi)jkl"), None);
        assert_eq!(find_group(&mut executor, ""), None);
        assert_eq!(find_group(&mut executor, "abcdef"), None);

        let mut executor = create_executor([(
            "def",
            Box::new(|_parameters: &Parameters| create_result("...")),
        )]);
        assert_eq!(
            find_group(&mut executor, "abc(def;ghi)jkl"),
            Some(("abc", "...".into(), "jkl"))
        );

        let mut executor = create_executor([(
            "",
            Box::new(|_parameters: &Parameters| create_result("...")),
        )]);
        assert_eq!(
            find_group(&mut executor, "abc(;ghi)jkl"),
            Some(("abc", "...".into(), "jkl"))
        );
        assert_eq!(
            find_group(&mut executor, "abc()jkl"),
            Some(("abc", "...".into(), "jkl"))
        );
        assert_eq!(
            find_group(&mut executor, "a)(b)c(de)f()jkl"),
            Some(("a)(b)c(de)f", "...".into(), "jkl"))
        );
    }
}
