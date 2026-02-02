use std::{fmt::Display, sync::Arc};

use convert_case::{Boundary, Case, Pattern};
use itertools::Itertools;

/// A structure that holds the name data of objects
#[derive(Debug, Clone, Eq, Default)]
pub struct Identifier {
    boundaries_applied: bool,
    /// The original string that was parsed without concats
    original: Arc<String>,
    words: Arc<[String]>,
    duplicate_id: Option<u32>,
}

impl Identifier {
    /// Try parse a string as an identifier.
    /// It will not have boundaries applied yet.
    pub fn try_parse(value: &str) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Self {
            boundaries_applied: false,
            original: Arc::new(value.into()),
            words: [value.into()].into(),
            duplicate_id: None,
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

        let mut words = Pattern::Lowercase.mutate(&words);
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

    pub fn check_validity(&self) -> Result<(), Error> {
        assert!(self.boundaries_applied);

        for (word_index, word) in self.words.iter().enumerate() {
            for (char_offset, char) in word.char_indices() {
                let tfn = match (word_index, char_offset) {
                    (0, 0) => |c| unicode_ident::is_xid_start(c) || c == '_',
                    _ => |c| unicode_ident::is_xid_continue(c),
                };

                if !tfn(char) {
                    // This is an option because I'm only 99% sure we can always find the word in the original
                    let offset = self
                        .original()
                        .to_lowercase()
                        .find(word)
                        .map(|word_offset| word_offset + char_offset);
                    return Err(Error::InvalidCharacter(offset, char));
                }
            }
        }

        Ok(())
    }

    /// Convert the identifier to a string in the given case
    pub fn to_case(&self, case: Case) -> String {
        assert!(
            self.boundaries_applied,
            "Boundaries not applied for `{}`",
            self.original()
        );

        let mut words = self.words.to_vec();

        if let Some(dup_id) = self.duplicate_id {
            words.push("dup".to_string());
            words.push(format!("{dup_id:X}"));
        }

        let words = case.mutate(&words.iter().map(|s| s.as_str()).collect_vec());
        case.join(&words)
    }

    /// Concatenate two identifiers together
    #[must_use]
    pub fn concat(self, rest: &Self) -> Self {
        let mut self_words = self.words.to_vec();
        let mut rest_words = rest.words.to_vec();
        self_words.append(&mut rest_words);
        Self {
            boundaries_applied: self.boundaries_applied && rest.boundaries_applied,
            original: Arc::new(self.original().to_owned() + "⧺" + rest.original()),
            words: self_words.into(),
            duplicate_id: self.duplicate_id,
        }
    }

    /// Get the original text. Don't use this unless it's important to get the *exact* original value.
    /// Better to use [`Self::to_case`] in most circumstances.
    pub fn original(&self) -> &str {
        &self.original
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|w| w.is_empty())
    }

    pub fn take_ref(&self) -> IdentifierRef {
        IdentifierRef {
            original: self.original.clone(),
        }
    }

    pub fn set_duplicate_id(&mut self, val: u32) {
        self.duplicate_id = Some(val);
    }

    pub fn duplicate_id(&self) -> Option<u32> {
        self.duplicate_id
    }
}

#[cfg(test)]
impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Identifier::try_parse(value).unwrap()
    }
}

impl std::hash::Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.original.hash(state);
        self.duplicate_id.hash(state);
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        (self.original == other.original || self.words == other.words)
            && self.duplicate_id == other.duplicate_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct IdentifierRef {
    original: Arc<String>,
}

impl IdentifierRef {
    pub fn new(identifier_original: String) -> Self {
        Self {
            original: Arc::new(identifier_original),
        }
    }

    pub fn original(&self) -> &str {
        &self.original
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Empty,
    InvalidCharacter(Option<usize>, char),
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Empty => write!(f, "Identifier is empty"),
            Error::InvalidCharacter(None, c) => {
                write!(f, "Identifier contains an invalid character: '{c:?}'")
            }
            Error::InvalidCharacter(Some(i), c) => {
                write!(
                    f,
                    "Identifier contains an invalid character at byte index {i}: '{c:?}'"
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
            Identifier::try_parse("1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter(Some(0), '1'))
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
            Identifier::try_parse("😈")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter(Some(0), '😈'))
        );
        assert_eq!(
            Identifier::try_parse("abc😈")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter(Some(3), '😈'))
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
            Identifier::try_parse("abc def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter(Some(3), ' '))
        );
        Identifier::try_parse("abc def")
            .unwrap()
            .apply_boundaries(&[Boundary::Space])
            .check_validity()
            .unwrap();
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
        assert_eq!(
            Identifier::try_parse("Bar🚩bar")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter(Some(3), '🚩'))
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

    #[test]
    fn default_is_empty() {
        assert!(Identifier::default().is_empty());
    }
}
