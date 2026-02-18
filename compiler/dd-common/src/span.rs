use std::{fmt::Display, ops::Range};

#[derive(Debug, Clone, Eq, PartialEq, Copy, Default, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Return self if not empty, or the other span if self is empty
    pub fn or(&self, other: Self) -> Self {
        if self.is_empty() { other } else { *self }
    }
}

impl chumsky::span::Span for Span {
    type Context = ();

    type Offset = usize;

    fn new(_context: Self::Context, range: Range<Self::Offset>) -> Self {
        range.into()
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl From<(usize, usize)> for Span {
    fn from(value: (usize, usize)) -> Self {
        assert!(value.0 <= value.1);

        Self {
            start: value.0,
            end: value.1,
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        assert!(value.start <= value.end);

        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

impl<'a> From<&'a Span> for Range<usize> {
    fn from(value: &'a Span) -> Self {
        value.start..value.end
    }
}

#[derive(Debug, Clone, Eq, Copy)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        // Only compare value. The span is transparent
        self.value == other.value
    }
}

impl<T: PartialEq> PartialEq<T> for Spanned<T> {
    fn eq(&self, other: &T) -> bool {
        // Only compare value. The span is transparent
        &self.value == other
    }
}

impl<T: std::hash::Hash> std::hash::Hash for Spanned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        // Only hash value. The span is transparent
    }
}

impl<T: Default> Default for Spanned<T> {
    fn default() -> Self {
        Self {
            span: (0, 0).into(),
            value: Default::default(),
        }
    }
}

impl<T: Display> Display for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Spanned<T> {
    pub fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }
}

impl<T, S: Into<Span>> From<(T, S)> for Spanned<T> {
    fn from((value, span): (T, S)) -> Self {
        Self {
            span: span.into(),
            value,
        }
    }
}

impl<T: PartialOrd> PartialOrd<T> for Spanned<T> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(other)
    }
}

pub trait SpanExt {
    fn with_span(self, span: impl Into<Span>) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned::new(span.into(), self)
    }

    /// Same as [Self::with_span], but can avoid name collisions
    fn spanned(self, span: impl Into<Span>) -> Spanned<Self>
    where
        Self: Sized,
    {
        self.with_span(span)
    }

    fn with_dummy_span(self) -> Spanned<Self>
    where
        Self: Sized,
    {
        self.with_span((0, 0))
    }

    fn into_with_dummy_span<T>(self) -> Spanned<T>
    where
        Self: Into<T>,
    {
        self.into().with_dummy_span()
    }
}
impl<T> SpanExt for T {}
