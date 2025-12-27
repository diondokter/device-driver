use std::{fmt::Display, sync::Arc};

use convert_case::{Boundary, Case};
use itertools::Itertools;

#[cfg(test)]
use crate::mir::Spanned;

/// A structure that holds the name data of objects
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Identifier {
    boundaries_applied: bool,
    /// The original string that was parsed without concats
    original: Arc<String>,
    words: Arc<[String]>,
}

impl Identifier {
    /// Try parse a string as an identifier.
    /// It will not have boundaries applied yet.
    pub fn try_parse(value: &str) -> Result<Self, Error> {
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

        Ok(Self {
            boundaries_applied: false,
            original: Arc::new(value.into()),
            words: [value.into()].into(),
        })
    }

    /// Apply the boundaries. This can only be called once and must be called before [`Self::to_case`]
    pub fn apply_boundaries(&mut self, boundaries: &[Boundary]) -> &mut Self {
        assert!(!self.boundaries_applied);

        let mut words = Vec::new();

        for word in self.words.iter() {
            let mut local_words = convert_case::split(word, boundaries);
            local_words.retain(|word| !word.is_empty());
            words.append(&mut local_words);
        }

        let mut words = words.into_iter().map(String::from).collect_vec();
        if boundaries.contains(&Boundary::Underscore) && self.original.starts_with('_') {
            match &mut words[..] {
                [] => words.push("_".into()),
                [word, ..] => *word = format!("_{word}"),
            }
        }

        self.boundaries_applied = true;
        self.words = words.into_iter().collect();
        self
    }

    /// Convert the identifier to a string in the given case
    pub fn to_case(&self, case: Case) -> String {
        assert!(self.boundaries_applied);

        let words = case.mutate(&self.words.iter().map(|w| w.as_str()).collect_vec());
        case.join(&words)
    }

    /// Concatenate two identifiers together
    #[must_use]
    pub fn concat(self, rest: &Self) -> Self {
        let mut self_words = self.words.to_vec();
        let mut rest_words = rest.words.to_vec();
        self_words.append(&mut rest_words);
        Self {
            boundaries_applied: true,
            original: self.original,
            words: self_words.into(),
        }
    }

    /// Get the original text. Don't use this unless it's important to get the *exact* original value.
    /// Better to use [`Self::to_case`] in most circumstances.
    pub fn original(&self) -> &str {
        &self.original
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|w| w.is_empty())
    }
}

#[cfg(test)]
impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Identifier::try_parse(value).unwrap()
    }
}

#[cfg(test)]
impl From<&str> for Spanned<Identifier> {
    fn from(value: &str) -> Self {
        use crate::mir::Span;

        Identifier::from(value).with_dummy_span()
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
        assert_eq!(Identifier::try_parse(""), Err(Error::Empty));
        assert_eq!(
            Identifier::try_parse("1"),
            Err(Error::InvalidCharacter(0, '1'))
        );
        assert_eq!(
            Identifier::try_parse("_1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "_1"
        );
        assert_eq!(
            Identifier::try_parse("a1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "a1"
        );
        assert_eq!(
            Identifier::try_parse("a_1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "a-1"
        );
        assert_eq!(
            Identifier::try_parse("😈"),
            Err(Error::InvalidCharacter(0, '😈'))
        );
        assert_eq!(
            Identifier::try_parse("abc😈"),
            Err(Error::InvalidCharacter(3, '😈'))
        );
        assert_eq!(
            Identifier::try_parse("_")
                .unwrap()
                .apply_boundaries(&[Boundary::Space])
                .to_case(Case::Kebab),
            "_"
        );
        assert_eq!(
            Identifier::try_parse("_")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "_"
        );
        assert_eq!(
            Identifier::try_parse("abc def"),
            Err(Error::InvalidCharacter(3, ' '))
        );
        assert_eq!(
            Identifier::try_parse("abc_def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "abc-def"
        );
        assert_eq!(
            Identifier::try_parse("_abc_def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "_abc-def"
        );
    }

    #[test]
    fn concat() {
        let mut id1 = Identifier::try_parse("abc_def").unwrap();
        id1.apply_boundaries(&[Boundary::Underscore]);
        let mut id2 = Identifier::try_parse("ghi_jkl").unwrap();
        id2.apply_boundaries(&[Boundary::Underscore]);

        let id3 = id1.concat(&id2);
        assert_eq!(id3.to_case(Case::Kebab), "abc-def-ghi-jkl");
    }
}
