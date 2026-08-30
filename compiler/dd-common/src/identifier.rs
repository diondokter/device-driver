use std::{
    fmt::{Debug, Display},
    num::NonZeroU32,
    sync::Arc,
};

use convert_case::{Boundary, Case, Pattern};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeNamespace {
    /// Used for things that participate in all namespaces.
    /// These are manifests, blocks, ...
    Global,
    /// Used for things that participate in no namespaces except the local definition.
    /// These are fields, enum variants, ...
    Local { site: Option<NonZeroU32> },
    /// Used for things that define operations or things you can do with a driver.
    /// These are registers, commands, buffers, ...
    Operation,
    /// Used for things that define a type.
    /// These are devices, fieldsets, enums, ...
    Type,
}

impl RuntimeNamespace {
    pub fn shares_namespace_with(&self, other: RuntimeNamespace) -> bool {
        for self_namespace in self.concrete_namespaces() {
            for other_namespace in other.concrete_namespaces() {
                if self_namespace == other_namespace {
                    return true;
                }
            }
        }

        false
    }

    pub fn concrete_namespaces(&self) -> Vec<RuntimeNamespace> {
        match self {
            RuntimeNamespace::Global => {
                vec![RuntimeNamespace::Operation, RuntimeNamespace::Type]
            }
            RuntimeNamespace::Local { .. } => vec![*self],
            RuntimeNamespace::Operation => vec![*self],
            RuntimeNamespace::Type => vec![*self],
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Type(RuntimeNamespace);
impl Default for Type {
    fn default() -> Self {
        Self(RuntimeNamespace::Type)
    }
}
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Operation(RuntimeNamespace);
impl Default for Operation {
    fn default() -> Self {
        Self(RuntimeNamespace::Operation)
    }
}
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Global(RuntimeNamespace);
impl Default for Global {
    fn default() -> Self {
        Self(RuntimeNamespace::Global)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Local(RuntimeNamespace);
impl Default for Local {
    fn default() -> Self {
        Self(RuntimeNamespace::Local { site: None })
    }
}

/// # Safety
/// Must only be implemented on type that are transparently [RuntimeType]
pub unsafe trait Namespace: Debug {
    fn runtime_value(&self) -> RuntimeNamespace;
}

unsafe impl Namespace for Type {
    fn runtime_value(&self) -> RuntimeNamespace {
        RuntimeNamespace::Type
    }
}
unsafe impl Namespace for Operation {
    fn runtime_value(&self) -> RuntimeNamespace {
        RuntimeNamespace::Operation
    }
}
unsafe impl Namespace for Global {
    fn runtime_value(&self) -> RuntimeNamespace {
        RuntimeNamespace::Global
    }
}
unsafe impl Namespace for Local {
    fn runtime_value(&self) -> RuntimeNamespace {
        self.0.runtime_value()
    }
}
unsafe impl Namespace for RuntimeNamespace {
    fn runtime_value(&self) -> RuntimeNamespace {
        *self
    }
}

impl From<Global> for Type {
    fn from(_: Global) -> Self {
        Type::default()
    }
}
impl From<Global> for Operation {
    fn from(_: Global) -> Self {
        Operation::default()
    }
}

/// A structure that holds the name data of objects
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct Identifier<T: Namespace> {
    boundaries_applied: bool,
    /// The original string that was parsed without concats
    original: Arc<String>,
    words: Arc<[String]>,
    duplicate_id: Option<NonZeroU32>,
    /// Must never change the internal runtime type!
    namespace: T,
}

impl<T: Namespace> Identifier<T> {
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
            namespace: id_type,
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

        if self.words.iter().all(String::is_empty) {
            return Err(Error::EmptyAfterSplits);
        }

        let converted = self.to_case(Case::Pascal);
        if converted.contains(['-', '_', ' ']) {
            return Err(Error::CannotConvert {
                case_name: "Pascal",
                example: converted,
            });
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

    /// Get the words derived from the original
    pub fn words(&self) -> Arc<[String]> {
        self.words.clone()
    }

    /// Get a display string that separates the words that make up the identifier visually
    pub fn words_display(&self) -> String {
        self.words.join("·")
    }

    /// Same as [Self::words_display], but prepends another word
    pub fn words_display_prepended(&self, word: String) -> String {
        let mut words = self.words.to_vec();
        words.insert(0, word);
        words.join("·")
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
            id_type: self.namespace.clone(),
        }
    }

    pub fn set_duplicate_id(&mut self, val: NonZeroU32) {
        self.duplicate_id = Some(val);
    }

    pub fn duplicate_id(&self) -> Option<NonZeroU32> {
        self.duplicate_id
    }

    pub fn to_runtime_namespace(self) -> Identifier<RuntimeNamespace> {
        Identifier {
            boundaries_applied: self.boundaries_applied,
            original: self.original,
            words: self.words,
            duplicate_id: self.duplicate_id,
            namespace: self.namespace.runtime_value(),
        }
    }

    pub fn as_runtime_namespace_mut(&mut self) -> &mut Identifier<RuntimeNamespace> {
        assert_eq!(size_of::<T>(), size_of::<RuntimeNamespace>());
        // Safety: We're only casting the T to a RuntimeType which is explicitly allowed by all implementors of IdentifierType
        // The Identifier itself is repr C and so won't be weird when the generic type changes
        unsafe { std::mem::transmute::<&mut Self, &mut Identifier<RuntimeNamespace>>(self) }
    }

    pub fn as_runtime_namespace(&self) -> &Identifier<RuntimeNamespace> {
        assert_eq!(size_of::<T>(), size_of::<RuntimeNamespace>());
        // Safety: We're only casting the T to a RuntimeType which is explicitly allowed by all implementors of IdentifierType
        // The Identifier itself is repr C and so won't be weird when the generic type changes
        unsafe { std::mem::transmute::<&Self, &Identifier<RuntimeNamespace>>(self) }
    }

    /// Get the identifier type
    pub fn namespace(&self) -> &T {
        &self.namespace
    }

    /// Change the type of the identifier to a more specific namespace
    pub fn cast<U>(self) -> Identifier<U>
    where
        U: Namespace + Default,
        U: From<T>,
    {
        // Fine to do since we have the where bound
        self.cast_unchecked()
    }

    /// Change the namespace of the identifier.
    /// This is generally a bad idea because of the subtleties!
    /// So make sure this is actually what you want.
    pub fn cast_unchecked<U: Namespace + Default>(self) -> Identifier<U> {
        Identifier {
            boundaries_applied: self.boundaries_applied,
            original: self.original,
            words: self.words,
            duplicate_id: self.duplicate_id,
            namespace: U::default(),
        }
    }

    /// Change the namespace of the identifier, but only if the runtime namespace is already that namespace.
    /// This function will panic if they're different.
    #[track_caller]
    pub fn cast_assert<U: Namespace + Default>(self) -> Identifier<U> {
        assert_eq!(self.namespace.runtime_value(), U::default().runtime_value());

        Identifier {
            boundaries_applied: self.boundaries_applied,
            original: self.original,
            words: self.words,
            duplicate_id: self.duplicate_id,
            namespace: U::default(),
        }
    }

    /// Returns true if the identifier can be safely compared to other identifiers
    pub fn is_valid(&self) -> bool {
        match self.namespace().runtime_value() {
            RuntimeNamespace::Global => true,
            RuntimeNamespace::Local { site } => site.is_some(),
            RuntimeNamespace::Operation => true,
            RuntimeNamespace::Type => true,
        }
    }
}

impl Identifier<Local> {
    pub fn set_local_site(&mut self, site: NonZeroU32) {
        self.namespace.0 = RuntimeNamespace::Local { site: Some(site) }
    }
}

impl Identifier<RuntimeNamespace> {
    /// Cast the namespace to a concrete namespace. Panics if the target is not a concrete namespace of the current namespace
    pub fn cast_concrete(
        self,
        runtime_namespace: RuntimeNamespace,
    ) -> Identifier<RuntimeNamespace> {
        assert!(
            self.namespace
                .runtime_value()
                .concrete_namespaces()
                .contains(&runtime_namespace)
        );

        Identifier {
            boundaries_applied: self.boundaries_applied,
            original: self.original,
            words: self.words,
            duplicate_id: self.duplicate_id,
            namespace: runtime_namespace,
        }
    }
}

impl<T: Namespace + Default> Default for Identifier<T> {
    fn default() -> Self {
        Self {
            boundaries_applied: Default::default(),
            original: Default::default(),
            words: Default::default(),
            duplicate_id: Default::default(),
            namespace: T::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct IdentifierRef<T: Namespace> {
    original: Arc<String>,
    id_type: T,
}

impl<T: Namespace> IdentifierRef<T> {
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

    pub fn is_ref_to<U: Namespace>(&self, identifier: &Identifier<U>) -> bool {
        identifier.namespace.runtime_value() == self.id_type.runtime_value()
            && self.original() == identifier.original()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Empty,
    EmptyAfterSplits,
    InvalidCharacter {
        byte_offset: usize,
        invalid_char: char,
    },
    CannotConvert {
        case_name: &'static str,
        example: String,
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
            Error::CannotConvert { case_name, example } => {
                write!(
                    f,
                    "cannot change the casing of the identifier. Identifier is `{example}` when converted to {case_name} case, but that's not correct casing"
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
        assert_eq!(Identifier::<Global>::try_parse(""), Err(Error::Empty));
        assert_eq!(
            Identifier::<Global>::try_parse("1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 0,
                invalid_char: '1'
            })
        );
        assert_eq!(
            Identifier::<Global>::try_parse("_1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "1"
        );
        assert_eq!(
            Identifier::<Global>::try_parse("a1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "a1"
        );
        assert_eq!(
            Identifier::<Global>::try_parse("a_1")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "a-1"
        );
        assert_eq!(
            Identifier::<Global>::try_parse("😈")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 0,
                invalid_char: '😈'
            })
        );
        assert_eq!(
            Identifier::<Global>::try_parse("abc😈")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 3,
                invalid_char: '😈'
            })
        );
        assert_eq!(
            Identifier::<Global>::try_parse("_")
                .unwrap()
                .apply_boundaries(&[Boundary::Space])
                .to_case(Case::Kebab),
            "_"
        );
        assert_eq!(
            Identifier::<Global>::try_parse("_")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::EmptyAfterSplits)
        );
        assert_eq!(
            Identifier::<Global>::try_parse("abc def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .check_validity(),
            Err(Error::InvalidCharacter {
                byte_offset: 3,
                invalid_char: ' '
            })
        );
        Identifier::<Global>::try_parse("abc def")
            .unwrap()
            .apply_boundaries(&[Boundary::Space])
            .check_validity()
            .unwrap();
        assert_eq!(
            Identifier::<Global>::try_parse("abc_def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "abc-def"
        );
        assert_eq!(
            Identifier::<Global>::try_parse("_abc_def")
                .unwrap()
                .apply_boundaries(&[Boundary::Underscore])
                .to_case(Case::Kebab),
            "abc-def"
        );
        assert_eq!(
            Identifier::<Global>::try_parse("Bar🚩bar")
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
        assert!(Identifier::<Global>::default().is_empty());
    }

    #[test]
    fn static_vs_runtime_equals() {
        assert_eq!(
            Identifier::<Type>::try_parse("a")
                .unwrap()
                .to_runtime_namespace(),
            Identifier::try_parse_with_type("a", RuntimeNamespace::Type).unwrap()
        );

        assert_ne!(
            Identifier::<Type>::try_parse("a")
                .unwrap()
                .to_runtime_namespace(),
            Identifier::try_parse_with_type("a", RuntimeNamespace::Operation).unwrap()
        );
    }

    #[test]
    fn issue_274() {
        // https://github.com/diondokter/device-driver/issues/274
        Identifier::<Global>::try_parse("io_pad_i2c_b1")
            .unwrap()
            .apply_boundaries(&Boundary::defaults())
            .check_validity()
            .unwrap();

        Identifier::<Global>::try_parse("io_pad_i2c-b1")
            .unwrap()
            .apply_boundaries(&[Boundary::Underscore])
            .check_validity()
            .unwrap_err();
    }
}
