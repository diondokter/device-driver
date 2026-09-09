use std::collections::HashMap;

use device_driver_diagnostics::Severity;
use tokio::sync::RwLock;
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    ls_types::{
        DiagnosticSeverity, DidOpenTextDocumentParams, InitializeResult, MessageType, Position,
        Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
    },
};

use crate::document::Document;

mod document;

pub struct Backend {
    client: Client,
    documents: RwLock<HashMap<Uri, Document>>,
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
            });
            Server::new(stdin, stdout, socket).serve(service).await;
        });
    }

    pub async fn update_document(&self, uri: Uri, source: String, version: i32) {
        let documents = self.documents.read().await;

        if let Some(document) = documents.get(&uri)
            && document.version() >= version
        {
            return;
        }

        drop(documents);

        match Document::compile(source, version) {
            Ok((document, diagnostics)) => {
                let diags = diagnostics
                    .iter()
                    .map(|diagnostic| {
                        let span = diagnostic.primary_span().as_line_column(document.source());

                        tower_lsp_server::ls_types::Diagnostic {
                            range: Range::new(
                                Position::new(span.0.0, span.0.1),
                                Position::new(span.1.0, span.1.1),
                            ),
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
                        }
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
    }
}

impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _params: tower_lsp_server::ls_types::InitializeParams,
    ) -> tower_lsp_server::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
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
        self.client
            .log_message(MessageType::LOG, format!("did_open: {params:?}"))
            .await;

        self.update_document(
            params.text_document.uri,
            params.text_document.text,
            params.text_document.version,
        )
        .await;
    }

    async fn did_change(&self, params: tower_lsp_server::ls_types::DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::LOG, format!("did_change: {params:?}"))
            .await;

        for change in params.content_changes {
            self.update_document(
                params.text_document.uri.clone(),
                change.text,
                params.text_document.version,
            )
            .await;
        }
    }

    async fn shutdown(&self) -> tower_lsp_server::jsonrpc::Result<()> {
        Ok(())
    }
}
