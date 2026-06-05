use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use convert_case::{Boundary, Case, Pattern};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeType {
    /// Used for things that participate in all types.
    /// These are manifests, blocks, fields, enum variants, ...
    All,
    /// Used for things that define operations or things you can do with a driver.
    /// These are registers, commands, buffers, ...
    Operation,
    /// Used for things that define a type.
    /// These are devices, fieldsets, enums, ...
    Type,
}

impl RuntimeType {
    pub fn shares_namespace_with(&self, other: RuntimeType) -> bool {
        matches!(
            (self, other),
            (RuntimeType::All, _)
                | (_, RuntimeType::All)
                | (RuntimeType::Operation, RuntimeType::Operation)
                | (RuntimeType::Type, RuntimeType::Type)
        )
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Type(RuntimeType);
impl Default for Type {
    fn default() -> Self {
        Self(RuntimeType::Type)
    }
}
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Operation(RuntimeType);
impl Default for Operation {
    fn default() -> Self {
        Self(RuntimeType::Operation)
    }
}
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct All(RuntimeType);
impl Default for All {
    fn default() -> Self {
        Self(RuntimeType::All)
    }
}

/// # Safety
/// Must only be implemented on type that are transparently [RuntimeType]
pub unsafe trait IdentifierType: Debug {
    fn runtime_value(&self) -> RuntimeType;
}

unsafe impl IdentifierType for Type {
    fn runtime_value(&self) -> RuntimeType {
        RuntimeType::Type
    }
}
unsafe impl IdentifierType for Operation {
    fn runtime_value(&self) -> RuntimeType {
        RuntimeType::Operation
    }
}
unsafe impl IdentifierType for All {
    fn runtime_value(&self) -> RuntimeType {
        RuntimeType::All
    }
}
unsafe impl IdentifierType for RuntimeType {
    fn runtime_value(&self) -> RuntimeType {
        *self
    }
}

impl From<All> for Type {
    fn from(_: All) -> Self {
        Type::default()
    }
}
impl From<All> for Operation {
    fn from(_: All) -> Self {
        Operation::default()
    }
}

/// A structure that holds the name data of objects
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Identifier<T: IdentifierType> {
    boundaries_applied: bool,
    /// The original string that was parsed without concats
    original: Arc<String>,
    words: Arc<[String]>,
    duplicate_id: Option<u32>,
    /// Must never change!
    id_type: T,
}

impl<T: IdentifierType> Identifier<T> {
    /// Try parse a string as an identifier.
    /// It will not have boundaries applied yet.
    pub fn try_parse(value: &str) -> Result<Self, Error>
    where
        T: Default,
    {
        Self::try_parse_with_type(value, T::default())
    }

    /// Try parse a string as an identifier.
    /// It will not have boundaries applied yet.
    pub fn try_parse_with_type(value: &str, id_type: T) -> Result<Self, Error> {
        if value.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Self {
            boundaries_applied: false,
            original: Arc::new(value.into()),
            words: [value.into()].into(),
            duplicate_id: None,
            id_type,
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

        let words = Pattern::Lowercase.mutate(&words);

        self.boundaries_applied = true;
        self.words = words.into_iter().collect();
        self
    }

    pub fn check_validity(&self) -> Result<(), Error> {
        assert!(self.boundaries_applied);

        for (word_index, word) in self.words.iter().enumerate() {
            for (char_offset, char) in word.char_indices() {
                let tfn = match (word_index, char_offset) {
                    (0, 0) => |c| unicode_ident::is_xid_start(c),
                    _ => |c| unicode_ident::is_xid_continue(c),
                };

                if !tfn(char) {
                    let offset = self
                        .original()
                        .to_lowercase()
                        .find(word)
                        .map(|word_offset| word_offset + char_offset)
                        .expect("Word should be present in identifier words");
                    return Err(Error::InvalidCharacter {
                        byte_offset: offset,
                        invalid_char: char,
                    });
                }
            }
        }

        if self.words().iter().all(String::is_empty) {
            return Err(Error::EmptyAfterSplits);
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

        let words = case.mutate(&words.iter().map(String::as_str).collect::<Vec<_>>());
        case.join(&words)
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
        self.words.iter().all(String::is_empty)
    }

    /// Get a type ref if this is a type identifier
    pub fn take_ref(&self) -> IdentifierRef<T>
    where
        T: Clone,
    {
        IdentifierRef {
            original: self.original.clone(),
            id_type: self.id_type.clone(),
        }
    }

    pub fn set_duplicate_id(&mut self, val: u32) {
        self.duplicate_id = Some(val);
    }

    pub fn duplicate_id(&self) -> Option<u32> {
        self.duplicate_id
    }

    pub fn to_runtime_type(self) -> Identifier<RuntimeType> {
        Identifier {
            boundaries_applied: self.boundaries_applied,
            original: self.original,
            words: self.words,
            duplicate_id: self.duplicate_id,
            id_type: self.id_type.runtime_value(),
        }
    }

    pub fn as_runtime_type_mut(&mut self) -> &mut Identifier<RuntimeType> {
        assert_eq!(size_of::<T>(), size_of::<RuntimeType>());
        // Safety: We're only casting the T to a RuntimeType which is explicitly allowed by all implementors of IdentifierType
        // The Identifier itself is repr C and so won't be weird when the generic type changes
        unsafe { std::mem::transmute::<&mut Self, &mut Identifier<RuntimeType>>(self) }
    }

    pub fn as_runtime_type(&self) -> &Identifier<RuntimeType> {
        assert_eq!(size_of::<T>(), size_of::<RuntimeType>());
        // Safety: We're only casting the T to a RuntimeType which is explicitly allowed by all implementors of IdentifierType
        // The Identifier itself is repr C and so won't be weird when the generic type changes
        unsafe { std::mem::transmute::<&Self, &Identifier<RuntimeType>>(self) }
    }

    /// Change the type of the identifier to a more specific type
    pub fn cast<U>(self) -> Identifier<U>
    where
        U: IdentifierType + Default,
        U: From<T>,
    {
        // Fine to do since we have the where bound
        self.cast_unchecked()
    }

    /// Change the type of the identifier.
    /// This is generally a bad idea because of the subtleties!
    /// So make sure this is actually what you want.
    pub fn cast_unchecked<U: IdentifierType + Default>(self) -> Identifier<U> {
        Identifier {
            boundaries_applied: self.boundaries_applied,
            original: self.original,
            words: self.words,
            duplicate_id: self.duplicate_id,
            id_type: U::default(),
        }
    }

    /// Change the type of the identifier, but only if the runtime type is already that type.
    /// This function will panic if they're different.
    #[track_caller]
    pub fn cast_assert<U: IdentifierType + Default>(self) -> Identifier<U> {
        assert_eq!(self.id_type.runtime_value(), U::default().runtime_value());

        Identifier {
            boundaries_applied: self.boundaries_applied,
            original: self.original,
            words: self.words,
            duplicate_id: self.duplicate_id,
            id_type: U::default(),
        }
    }
}

impl<T: IdentifierType + Default> Default for Identifier<T> {
    fn default() -> Self {
        Self {
            boundaries_applied: Default::default(),
            original: Default::default(),
            words: Default::default(),
            duplicate_id: Default::default(),
            id_type: T::default(),
        }
    }
}

impl<T: IdentifierType> std::hash::Hash for Identifier<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.original.hash(state);
        self.duplicate_id.hash(state);
        self.id_type.runtime_value().hash(state);
    }
}

impl<T: IdentifierType> PartialEq for Identifier<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.original == other.original || self.words == other.words)
            && self.duplicate_id == other.duplicate_id
            && self
                .id_type
                .runtime_value()
                .shares_namespace_with(other.id_type.runtime_value())
    }
}
impl<T: IdentifierType> Eq for Identifier<T> {}

#[derive(Debug, Clone, Default)]
pub struct IdentifierRef<T: IdentifierType> {
    original: Arc<String>,
    id_type: T,
}

impl<T: IdentifierType> IdentifierRef<T> {
    pub fn new(identifier_original: String) -> Self
    where
        T: Default,
    {
        Self {
            original: Arc::new(identifier_original),
            id_type: T::default(),
        }
    }

    pub fn original(&self) -> &str {
        &self.original
    }

    pub fn is_ref_to<U: IdentifierType>(&self, identifier: &Identifier<U>) -> bool {
        identifier.id_type.runtime_value() == self.id_type.runtime_value()
            && self.original() == identifier.original()
    }
}

impl<T: IdentifierType> std::hash::Hash for IdentifierRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.original.hash(state);
        self.id_type.runtime_value().hash(state);
    }
}

impl<T: IdentifierType> PartialEq for IdentifierRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.original == other.original
            && self
                .id_type
                .runtime_value()
                .shares_namespace_with(other.id_type.runtime_value())
    }
}
impl<T: IdentifierType> Eq for IdentifierRef<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Empty,
    EmptyAfterSplits,
    InvalidCharacter {
        byte_offset: usize,
        invalid_char: char,
    },
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Empty => write!(f, "identifier is empty"),
            Error::EmptyAfterSplits => write!(f, "identifier is empty after word split"),
            Error::InvalidCharacter {
                byte_offset,
                invalid_char,
            } => {
                write!(
                    f,
                    "identifier contains an invalid character at byte offset {byte_offset}: '{invalid_char:?}'"
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
        assert_eq!(Identifier::<All>::try_parse(""), Err(Error::Empty));
        assert_eq!(
            Identifier::<All>::try_parse("1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 0,
                invalid_char: '1'
            })
        );
        assert_eq!(
            Identifier::<All>::try_parse("_1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "1"
        );
        assert_eq!(
            Identifier::<All>::try_parse("a1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "a1"
        );
        assert_eq!(
            Identifier::<All>::try_parse("a_1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "a-1"
        );
        assert_eq!(
            Identifier::<All>::try_parse("😈")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 0,
                invalid_char: '😈'
            })
        );
        assert_eq!(
            Identifier::<All>::try_parse("abc😈")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 3,
                invalid_char: '😈'
            })
        );
        assert_eq!(
            Identifier::<All>::try_parse("_")
                .unwrap()
                .apply_boundaries(&[Boundary::Space])
                .to_case(Case::Kebab),
            "_"
        );
        assert_eq!(
            Identifier::<All>::try_parse("_")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::EmptyAfterSplits)
        );
        assert_eq!(
            Identifier::<All>::try_parse("abc def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 3,
                invalid_char: ' '
            })
        );
        Identifier::<All>::try_parse("abc def")
            .unwrap()
            .apply_boundaries(&[Boundary::Space])
            .check_validity()
            .unwrap();
        assert_eq!(
            Identifier::<All>::try_parse("abc_def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "abc-def"
        );
        assert_eq!(
            Identifier::<All>::try_parse("_abc_def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "abc-def"
        );
        assert_eq!(
            Identifier::<All>::try_parse("Bar🚩bar")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 3,
                invalid_char: '🚩'
            })
        );
    }

    #[test]
    fn default_is_empty() {
        assert!(Identifier::<All>::default().is_empty());
    }

    #[test]
    fn static_vs_runtime_equals() {
        assert_eq!(
            Identifier::<Type>::try_parse("a")
                .unwrap()
                .to_runtime_type(),
            Identifier::try_parse_with_type("a", RuntimeType::Type).unwrap()
        );

        assert_ne!(
            Identifier::<Type>::try_parse("a")
                .unwrap()
                .to_runtime_type(),
            Identifier::try_parse_with_type("a", RuntimeType::Operation).unwrap()
        );
    }

    #[test]
    fn all_vs_specific_equals() {
        assert_eq!(
            Identifier::<All>::try_parse("a").unwrap().to_runtime_type(),
            Identifier::<Type>::try_parse("a")
                .unwrap()
                .to_runtime_type(),
        );
        assert_eq!(
            Identifier::<All>::try_parse("a").unwrap().to_runtime_type(),
            Identifier::<Operation>::try_parse("a")
                .unwrap()
                .to_runtime_type(),
        );
    }
}
