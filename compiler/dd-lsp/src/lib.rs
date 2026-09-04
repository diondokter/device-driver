use camino::{Utf8Path, Utf8PathBuf};
use device_driver_diagnostics::{Diagnostics, DynError, Metadata, ResultExt};
use tower_lsp_server::{
    Client, LanguageServer, LspService, Server,
    ls_types::{
        DiagnosticSeverity, DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeResult,
        MessageType, Position, Range, ServerCapabilities, TextDocumentSyncCapability,
        TextDocumentSyncKind, Uri,
    },
};

#[derive(Debug)]
pub struct Backend {
    client: Client,
}

impl Backend {
    pub fn run() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        rt.block_on(async {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();
            let (service, socket) = LspService::new(|client| Backend { client });
            Server::new(stdin, stdout, socket).serve(service).await;
        });
    }

    pub fn try_compile(&self, source: &str) -> Result<Diagnostics, DynError> {
        let mut diagnostics = Diagnostics::new();

        let tokens = device_driver_lexer::lex(source);
        let ast = device_driver_parser::parse(&tokens, &mut diagnostics);
        let _mir = device_driver_mir::lower_ast(ast, &Default::default(), &mut diagnostics)
            .with_message(|| "lower ast into MIR")?;

        Ok(diagnostics)
    }

    pub async fn compile(&self, file: &Utf8Path, source: &str, version: Option<i32>) {
        match self.try_compile(source) {
            Ok(diagnostics) => {
                let mut diags = Vec::new();

                let Ok(()) = diagnostics.render_for_each(
                    Metadata {
                        source,
                        source_path: file.as_str(),
                        term_width: None,
                        ansi: false,
                        unicode: false,
                        anonymized_line_numbers: false,
                    },
                    |diagnostic, rendered| {
                        let span = diagnostic.main_span().as_line_column(source);

                        diags.push(tower_lsp_server::ls_types::Diagnostic {
                            range: Range::new(
                                Position::new(span.0.0, span.0.1),
                                Position::new(span.1.0, span.1.1),
                            ),
                            severity: if diagnostic.is_error() {
                                Some(DiagnosticSeverity::ERROR)
                            } else {
                                Some(DiagnosticSeverity::WARNING)
                            },
                            code: None,
                            code_description: None,
                            source: Some("DDSL".into()),
                            message: rendered,
                            related_information: None, // TODO: look at
                            tags: None,
                            data: None,
                        });
                        Result::<(), std::convert::Infallible>::Ok(())
                    },
                );

                self.client
                    .publish_diagnostics(Uri::from_file_path(file).unwrap(), diags, version)
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
        eprintln!("did_open");

        let path = Utf8PathBuf::from(params.text_document.uri.path().to_string())
            .canonicalize_utf8()
            .unwrap();

        self.client
            .log_message(MessageType::LOG, format!("did_open: {}", path))
            .await;

        self.compile(
            &path,
            &params.text_document.text,
            Some(params.text_document.version),
        )
        .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let path = Utf8PathBuf::from(params.text_document.uri.path().to_string())
            .canonicalize_utf8()
            .unwrap();

        self.client
            .log_message(MessageType::LOG, format!("did_save: {}", path))
            .await;

        self.compile(&path, params.text.as_ref().unwrap(), None)
            .await;
    }

    async fn did_change(&self, params: tower_lsp_server::ls_types::DidChangeTextDocumentParams) {
        let path = Utf8PathBuf::from(params.text_document.uri.path().to_string())
            .canonicalize_utf8()
            .unwrap();

        self.client
            .log_message(MessageType::LOG, format!("did_change: {}", path))
            .await;

        for change in params.content_changes {
            self.compile(&path, &change.text, None).await;
        }
    }

    async fn shutdown(&self) -> tower_lsp_server::jsonrpc::Result<()> {
        Ok(())
    }
}
