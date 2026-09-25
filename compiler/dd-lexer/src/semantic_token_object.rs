use device_driver_common::{
    span::{Span, SpanExt, Spanned},
    specifiers::{Access, AddressMode, BaseType, ByteOrder, Integer},
};

use crate::{Token, TokenModifier, TokenType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticToken<'src> {
    pub token: Token<'src>,
    pub semantic_type: Option<TokenType>,
    pub modifiers: u32,
}

impl<'src> Token<'src> {
    pub fn without_semantics(self) -> SemanticToken<'src> {
        SemanticToken {
            token: self,
            semantic_type: None,
            modifiers: 0,
        }
    }

    pub fn with_semantics(
        self,
        semantic_type: TokenType,
        modifiers: &[TokenModifier],
    ) -> SemanticToken<'src> {
        SemanticToken {
            token: self,
            semantic_type: Some(semantic_type),
            modifiers: modifiers.iter().map(|modifier| *modifier as u32).sum(),
        }
    }
}

/// An object that can be turned into semantic tokens
pub trait SemanticTokenObject {
    type Context;

    fn to_tokens_in(&self, ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>);
    fn to_tokens(&self, ctx: &Self::Context) -> Vec<SemanticToken<'_>> {
        let mut tokens = Vec::new();
        self.to_tokens_in(ctx, &mut tokens);
        tokens
    }
    fn to_respanned_tokens<'src>(
        &self,
        source: &'src str,
        object_span: Span,
        ctx: &Self::Context,
    ) -> Vec<Spanned<SemanticToken<'src>>> {
        let self_tokens = self.to_tokens(ctx);
        let mut self_index = 0;

        let source_tokens = super::lex(&source[std::ops::Range::from(object_span)]);
        let mut semantic_source_tokens = Vec::with_capacity(source_tokens.len());

        let ignore = |token: &Token| matches!(token, Token::Comma);

        for mut source_token in source_tokens.into_iter() {
            source_token.span.start += object_span.start;
            source_token.span.end += object_span.start;

            if !ignore(&source_token) {
                while let Some(self_token) = self_tokens.get(self_index)
                    && ignore(&self_token.token)
                {
                    self_index += 1;
                }

                let semantic_token = if let Some(self_token) = self_tokens.get(self_index) {
                    SemanticToken {
                        token: source_token.value,
                        semantic_type: self_token.semantic_type,
                        modifiers: self_token.modifiers,
                    }
                    .with_span(source_token.span)
                } else {
                    source_token
                        .without_semantics()
                        .with_span(source_token.span)
                };

                semantic_source_tokens.push(semantic_token);

                self_index += 1;
            } else {
                semantic_source_tokens.push(
                    source_token
                        .value
                        .without_semantics()
                        .with_span(source_token.span),
                );
            }
        }

        semantic_source_tokens
    }
}

impl SemanticTokenObject for Access {
    type Context = ();

    fn to_tokens_in(&self, _ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.push(Token::Access(*self).with_semantics(TokenType::Access, &[]));
    }
}

impl SemanticTokenObject for ByteOrder {
    type Context = ();

    fn to_tokens_in(&self, _ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.push(Token::ByteOrder(*self).with_semantics(TokenType::ByteOrder, &[]));
    }
}

impl SemanticTokenObject for BaseType {
    type Context = ();

    fn to_tokens_in(&self, _ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.push(
            Token::BaseType(*self)
                .with_semantics(TokenType::Type, &[TokenModifier::DefaultLibrary]),
        );
    }
}

impl SemanticTokenObject for Integer {
    type Context = ();

    fn to_tokens_in(&self, _ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.push(
            Token::Integer(*self).with_semantics(TokenType::Type, &[TokenModifier::DefaultLibrary]),
        );
    }
}

impl SemanticTokenObject for AddressMode {
    type Context = ();

    fn to_tokens_in(&self, _ctx: &Self::Context, tokens: &mut Vec<SemanticToken<'_>>) {
        tokens.push(Token::AddressMode(*self).with_semantics(TokenType::AddressMode, &[]));
    }
}
