use device_driver_diagnostics::{Diagnostics, DynError, ResultExt};
use device_driver_parser::Ast;

pub struct Document {
    version: i32,
    source: String,
    ast: Ast,
}

impl Document {
    pub fn compile(source: String, version: i32) -> Result<(Self, Diagnostics), DynError> {
        let mut diagnostics = Diagnostics::new();

        let tokens = device_driver_lexer::lex(&source);
        let ast = device_driver_parser::parse(&tokens, &mut diagnostics);
        let _mir = device_driver_mir::lower_ast(&ast, &Default::default(), &mut diagnostics)
            .with_message(|| "lower ast into MIR")?;

        Ok((
            Document {
                version,
                source,
                ast,
            },
            diagnostics,
        ))
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn version(&self) -> i32 {
        self.version
    }
}
