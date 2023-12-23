pub enum ProgramPart {
    RawString(String),
    SubstitutionInvocation(Vec<ProgramPart>),
}

fn split(s: &str) -> Option<(char, &str)> {
    let mut s = s.chars();
    s.next().map(|c| (c, s.as_str()))
}

type ParsingResult<T> = parco::Result<T, &str, ()>;

fn parse_raw_string_char(rest: &str) -> ParsingResult<char> {
    
    parco::one_matching_part(rest, |c| "();".contains(c)).map(|_| ParsingResult::Err)
}

fn parse_raw_string(mut program: &str) -> String {
    let mut result = String::new();
    while let Some((c, rest)) = program {
        result.push(match c {
            '\\' => match split(rest) {
                None => {
                    program = rest;
                    '\\'
                },
                Some((c, rest)) => {
                    program = rest;
                    c
                }
            },
            '(' | ')' | ';' => {
                break;
            },
            c => {
                program = rest;
                c
            },
        });
        program = rest;
    }
    result.shrink_to_fit();
    result
}

fn parse_

pub fn parse(string: &str) -> ProgramPart {

}

pub struct PatternRule {
    regex: regex::Regex,
}

enum PatternPart {
    VerbatimChar(char),
    AnyCharacter,
    AnySequence,
}

pub enum PatternCreationError {
    AdjacentStars,
    StarAtBeginning,
}

pub enum MatchingError {
    NotMatched,
}

impl PatternRule {
    pub fn new(s: &str) -> Result<Self, PatternCreationError> {

    }

    pub fn match(&self, s: &str) -> Result<, MatchingError>
}
