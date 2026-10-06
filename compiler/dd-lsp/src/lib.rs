use std::{collections::HashMap, sync::OnceLock};

use device_driver_common::{instant::Instant, specifiers::VariantNames};
use device_driver_diagnostics::Severity;
use device_driver_lexer::{TokenModifier, TokenType};
use tokio::sync::RwLock;
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    jsonrpc::Error,
    ls_types::{
        ClientCapabilities, DeclarationCapability, DiagnosticSeverity, DidOpenTextDocumentParams,
        DocumentSymbolResponse, GotoDefinitionResponse, InitializeResult, InlayHint, Location,
        MessageType, OneOf, PrepareRenameResponse, RenameOptions, SemanticTokensFullOptions,
        SemanticTokensLegend, SemanticTokensOptions, SemanticTokensRangeResult,
        SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities,
        TextDocumentSyncCapability, TextDocumentSyncKind, Uri, WorkDoneProgressOptions,
    },
};

use crate::document::Document;

mod document;
mod document_symbol;
mod goto_definition;
mod inlay_hints;
mod rename;
mod semantic_tokens;
mod util;

pub struct Backend {
    client: Client,
    documents: RwLock<HashMap<Uri, Document>>,
    client_capabilities: OnceLock<ClientCapabilities>,
}

impl Backend {
    pub fn run() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        rt.block_on(async {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();
            let (service, socket) = LspService::new(|client| Backend {
                client,
                documents: RwLock::new(HashMap::new()),
                client_capabilities: Default::default(),
            });
            Server::new(stdin, stdout, socket).serve(service).await;
        });
    }

    pub async fn update_document(&self, uri: Uri, source: String, version: i32) {
        let start = Instant::now();

        let documents = self.documents.read().await;

        if let Some(document) = documents.get(&uri)
            && document.version() >= version
        {
            return;
        }

        drop(documents);

        match Document::compile(source, version, uri.clone()) {
            Ok((document, diagnostics)) => {
                let diags = diagnostics
                    .iter()
                    .map(|diagnostic| tower_lsp_server::ls_types::Diagnostic {
                        range: document.translate_span(diagnostic.primary_span()),
                        severity: match diagnostic.severity() {
                            Severity::Error => Some(DiagnosticSeverity::ERROR),
                            Severity::Warning => Some(DiagnosticSeverity::WARNING),
                            Severity::Info => Some(DiagnosticSeverity::INFORMATION),
                            Severity::Note => Some(DiagnosticSeverity::HINT),
                            Severity::Help => Some(DiagnosticSeverity::HINT),
                        },
                        code: None,
                        code_description: None,
                        source: Some("DDSL".into()),
                        message: diagnostic.title().into(),
                        related_information: None,
                        tags: None,
                        data: None,
                    })
                    .collect();

                self.documents.write().await.insert(uri.clone(), document);

                self.client
                    .publish_diagnostics(uri, diags, Some(version))
                    .await;
            }
            Err(e) => {
                self.client.log_message(MessageType::ERROR, e).await;
            }
        }

        let elapsed = start.elapsed();
        self.client
            .log_message(
                MessageType::LOG,
                format!("update_document took {}ms", elapsed.as_secs_f32() * 1000.0),
            )
            .await;
    }
}

impl LanguageServer for Backend {
    async fn initialize(
        &self,
        params: tower_lsp_server::ls_types::InitializeParams,
    ) -> tower_lsp_server::jsonrpc::Result<InitializeResult> {
        let _ = self.client_capabilities.set(params.capabilities);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_symbol_provider: Some(OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            work_done_progress_options: Default::default(),
                            legend: SemanticTokensLegend {
                                token_types: TokenType::VARIANTS
                                    .iter()
                                    .map(|tt| (*tt).into())
                                    .collect(),
                                token_modifiers: TokenModifier::VARIANTS
                                    .iter()
                                    .map(|tm| (*tm).into())
                                    .collect(),
                            },
                            range: Some(true),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                        },
                    ),
                ),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                definition_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(tower_lsp_server::ls_types::ServerInfo {
                name: env!("CARGO_PKG_NAME").into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _params: tower_lsp_server::ls_types::InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_document(
            params.text_document.uri,
            params.text_document.text,
            params.text_document.version,
        )
        .await;
    }

    async fn did_change(&self, params: tower_lsp_server::ls_types::DidChangeTextDocumentParams) {
        for change in params.content_changes {
            self.update_document(
                params.text_document.uri.clone(),
                change.text,
                params.text_document.version,
            )
            .await;
        }
    }

    async fn did_close(&self, params: tower_lsp_server::ls_types::DidCloseTextDocumentParams) {
        // Remove the document from memory
        self.documents
            .write()
            .await
            .remove(&params.text_document.uri);
    }

    async fn document_symbol(
        &self,
        params: tower_lsp_server::ls_types::DocumentSymbolParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<DocumentSymbolResponse>> {
        let start = Instant::now();

        let guard = self.documents.read().await;
        let Some(document) = guard.get(&params.text_document.uri) else {
            return Err(Error::invalid_params(params.text_document.uri.to_string()));
        };

        let Some(root_node) = document.ast().root_node else {
            return Ok(None);
        };

        let root_node_symbol =
            document_symbol::get_node_symbol(document.ast().node(root_node), document);

        let elapsed = start.elapsed();
        self.client
            .log_message(
                MessageType::LOG,
                format!("document_symbol took {}ms", elapsed.as_secs_f32() * 1000.0),
            )
            .await;

        Ok(Some(DocumentSymbolResponse::Nested(vec![root_node_symbol])))
    }

    async fn inlay_hint(
        &self,
        params: tower_lsp_server::ls_types::InlayHintParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<Vec<InlayHint>>> {
        let start = Instant::now();

        let guard = self.documents.read().await;
        let Some(document) = guard.get(&params.text_document.uri) else {
            return Err(Error::invalid_params(params.text_document.uri.to_string()));
        };

        let hints = inlay_hints::get_hints(
            document.ast(),
            document.translate_range(params.range),
            document,
        );

        let elapsed = start.elapsed();
        self.client
            .log_message(
                MessageType::LOG,
                format!("inlay_hint took {}ms", elapsed.as_secs_f32() * 1000.0),
            )
            .await;

        Ok(Some(hints))
    }

    async fn semantic_tokens_full(
        &self,
        params: tower_lsp_server::ls_types::SemanticTokensParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<SemanticTokensResult>> {
        let start = Instant::now();

        let guard = self.documents.read().await;
        let Some(document) = guard.get(&params.text_document.uri) else {
            return Err(Error::invalid_params(params.text_document.uri.to_string()));
        };

        let Some(root_node) = document.ast().root_node else {
            return Ok(None);
        };

        let tokens = semantic_tokens::calculate_semantic_tokens(root_node, document, None);

        let elapsed = start.elapsed();
        self.client
            .log_message(
                MessageType::LOG,
                format!(
                    "semantic_tokens_full took {}ms",
                    elapsed.as_secs_f32() * 1000.0
                ),
            )
            .await;

        Ok(Some(SemanticTokensResult::Tokens(tokens)))
    }

    async fn semantic_tokens_range(
        &self,
        params: tower_lsp_server::ls_types::SemanticTokensRangeParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<SemanticTokensRangeResult>> {
        let start = Instant::now();

        let guard = self.documents.read().await;
        let Some(document) = guard.get(&params.text_document.uri) else {
            return Err(Error::invalid_params(params.text_document.uri.to_string()));
        };

        let Some(root_node) = document.ast().root_node.as_ref() else {
            return Ok(None);
        };

        let tokens =
            semantic_tokens::calculate_semantic_tokens(*root_node, document, Some(params.range));

        let elapsed = start.elapsed();
        self.client
            .log_message(
                MessageType::LOG,
                format!(
                    "semantic_tokens_range took {}ms",
                    elapsed.as_secs_f32() * 1000.0
                ),
            )
            .await;

        Ok(Some(SemanticTokensRangeResult::Tokens(tokens)))
    }

    async fn prepare_rename(
        &self,
        params: tower_lsp_server::ls_types::TextDocumentPositionParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<PrepareRenameResponse>> {
        let start = Instant::now();

        let guard = self.documents.read().await;
        let Some(document) = guard.get(&params.text_document.uri) else {
            return Err(Error::invalid_params(params.text_document.uri.to_string()));
        };

        let prep_rename_result = rename::find_rename_target(document, params.position);

        let elapsed = start.elapsed();
        self.client
            .log_message(
                MessageType::LOG,
                format!("prepare_rename took {}ms", elapsed.as_secs_f32() * 1000.0,),
            )
            .await;

        match prep_rename_result {
            Ok((range, placeholder)) => Ok(Some(PrepareRenameResponse::RangeWithPlaceholder {
                range,
                placeholder: placeholder.original().to_string(),
            })),
            Err(message) => Err(Error::invalid_params(message)),
        }
    }

    async fn rename(
        &self,
        params: tower_lsp_server::ls_types::RenameParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<tower_lsp_server::ls_types::WorkspaceEdit>> {
        let start = Instant::now();

        let guard = self.documents.read().await;
        let Some(document) = guard.get(&params.text_document_position.text_document.uri) else {
            return Err(Error::invalid_params(
                params.text_document_position.text_document.uri.to_string(),
            ));
        };

        let edit = rename::rename(
            document,
            params.text_document_position.position,
            params.new_name,
        );

        let elapsed = start.elapsed();
        self.client
            .log_message(
                MessageType::LOG,
                format!("rename took {}ms", elapsed.as_secs_f32() * 1000.0),
            )
            .await;

        match edit {
            Ok(edit) => Ok(Some(edit)),
            Err(message) => Err(Error::invalid_params(message)),
        }
    }

    async fn goto_declaration(
        &self,
        params: tower_lsp_server::ls_types::request::GotoDeclarationParams,
    ) -> tower_lsp_server::jsonrpc::Result<
        Option<tower_lsp_server::ls_types::request::GotoDeclarationResponse>,
    > {
        self.goto_definition(params).await
    }

    async fn goto_definition(
        &self,
        params: tower_lsp_server::ls_types::GotoDefinitionParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let start = Instant::now();

        let guard = self.documents.read().await;
        let Some(document) = guard.get(&params.text_document_position_params.text_document.uri)
        else {
            return Err(Error::invalid_params(
                params
                    .text_document_position_params
                    .text_document
                    .uri
                    .to_string(),
            ));
        };

        let definition = goto_definition::goto_definition(
            document,
            params.text_document_position_params.position,
        );

        let link_support = self
            .client_capabilities
            .get()
            .and_then(|caps| caps.text_document.as_ref())
            .and_then(|txt_doc| txt_doc.definition)
            .and_then(|def| def.link_support)
            .unwrap_or(false);

        let result = if link_support {
            definition.map(|location| GotoDefinitionResponse::Link(vec![location]))
        } else {
            definition.map(|location| {
                GotoDefinitionResponse::Scalar(Location {
                    uri: location.target_uri,
                    range: location.target_range,
                })
            })
        };

        let elapsed = start.elapsed();
        self.client
            .log_message(
                MessageType::LOG,
                format!("goto_definition took {}ms", elapsed.as_secs_f32() * 1000.0),
            )
            .await;

        Ok(result)
    }

    async fn shutdown(&self) -> tower_lsp_server::jsonrpc::Result<()> {
        Ok(())
    }
}
