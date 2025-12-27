use std::{fmt::Display, rc::Rc};

use convert_case::{Boundary, Case};
use itertools::Itertools;

/// A strucure that holds the name data of objects
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Identifier {
    original: String,
    words: Rc<[String]>,
}

impl Identifier {
    /// Try parse a string as an identifier
    pub fn try_parse(value: &str, boundaries: Option<&[Boundary]>) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::Empty);
        }

        for (i, char) in value.chars().enumerate() {
            let tfn = match i {
                0 => |c| unicode_ident::is_xid_start(c) || c == '_',
                _ => |c| unicode_ident::is_xid_continue(c),
            };

            if !tfn(char) {
                return Err(Error::InvalidCharacter(i, char));
            }
        }

        let default_boundaries = Boundary::defaults();
        let boundaries = boundaries.unwrap_or(&default_boundaries);
        let mut words = convert_case::split(&value, boundaries);
        words.retain(|word| !word.is_empty());
        let mut words = words.into_iter().map(String::from).collect_vec();
        if boundaries.contains(&Boundary::Underscore) && value.starts_with('_') {
            match &mut words[..] {
                [] => words.push("_".into()),
                [word, ..] => *word = format!("_{word}"),
            }
        }

        Ok(Self {
            original: value.into(),
            words: words.into_iter().collect(),
        })
    }

    /// Convert the identifier to a string in the given case
    pub fn to_case(&self, case: Case) -> String {
        let words = case.mutate(&self.words.iter().map(|w| w.as_str()).collect_vec());
        case.join(&words)
    }

    /// Concatenate two identifiers together
    #[must_use]
    pub fn concat(self, rest: Self) -> Self {
        let mut self_words = self.words.to_vec();
        let mut rest_words = rest.words.to_vec();
        self_words.append(&mut rest_words);
        Self {
            original: self.original + &rest.original,
            words: self_words.into(),
        }
    }

    /// Get the original text. Don't use this unless it's important to get the *exact* original value.
    /// Better to use [`Self::to_case`] in most circumstances.
    pub fn original(&self) -> &str {
        &self.original
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Empty,
    InvalidCharacter(usize, char),
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Empty => write!(f, "Identifier is empty"),
            Error::InvalidCharacter(i, c) => {
                write!(
                    f,
                    "Identifier contains an invalid character at index {i}: '{c:?}'"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_cases() {
        assert_eq!(Identifier::try_parse("", None), Err(Error::Empty));
        assert_eq!(
            Identifier::try_parse("1", None),
            Err(Error::InvalidCharacter(0, '1'))
        );
        assert_eq!(
            Identifier::try_parse("_1", Some(&[Boundary::Underscore]))
                .unwrap()
                .to_case(Case::Kebab),
            "_1"
        );
        assert_eq!(
            Identifier::try_parse("a1", Some(&[Boundary::Underscore]))
                .unwrap()
                .to_case(Case::Kebab),
            "a1"
        );
        assert_eq!(
            Identifier::try_parse("a_1", Some(&[Boundary::Underscore]))
                .unwrap()
                .to_case(Case::Kebab),
            "a-1"
        );
        assert_eq!(
            Identifier::try_parse("😈", None),
            Err(Error::InvalidCharacter(0, '😈'))
        );
        assert_eq!(
            Identifier::try_parse("abc😈", None),
            Err(Error::InvalidCharacter(3, '😈'))
        );
        assert_eq!(
            Identifier::try_parse("_", Some(&[Boundary::Space]))
                .unwrap()
                .to_case(Case::Kebab),
            "_"
        );
        assert_eq!(
            Identifier::try_parse("_", Some(&[Boundary::Underscore]))
                .unwrap()
                .to_case(Case::Kebab),
            "_"
        );
        assert_eq!(
            Identifier::try_parse("abc def", None),
            Err(Error::InvalidCharacter(3, ' '))
        );
        assert_eq!(
            Identifier::try_parse("abc_def", Some(&[Boundary::Underscore]))
                .unwrap()
                .to_case(Case::Kebab),
            "abc-def"
        );
        assert_eq!(
            Identifier::try_parse("_abc_def", Some(&[Boundary::Underscore]))
                .unwrap()
                .to_case(Case::Kebab),
            "_abc-def"
        );
    }

    #[test]
    fn concat() {
        let id1 = Identifier::try_parse("abc_def", Some(&[Boundary::Underscore])).unwrap();
        let id2 = Identifier::try_parse("ghi_jkl", Some(&[Boundary::Underscore])).unwrap();

        let id3 = id1.concat(id2);
        assert_eq!(id3.to_case(Case::Kebab), "abc-def-ghi-jkl");
    }
}
