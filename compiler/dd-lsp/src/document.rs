use device_driver_diagnostics::{Diagnostics, DynError, ResultExt};
use device_driver_mir::model::Manifest;
use device_driver_parser::Ast;

pub struct Document {
    version: i32,
    source: String,
    ast: Ast,
    mir: Manifest,
}

impl Document {
    pub fn compile(source: String, version: i32) -> Result<(Self, Diagnostics), DynError> {
        let mut diagnostics = Diagnostics::new();

        let tokens = device_driver_lexer::lex(&source);
        let ast = device_driver_parser::parse(&tokens, &mut diagnostics);
        let (mir, _) = device_driver_mir::lower_ast(&ast, &Default::default(), &mut diagnostics)
            .with_message(|| "lower ast into MIR")?;

        Ok((
            Document {
                version,
                source,
                ast,
                mir,
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

    pub fn mir(&self) -> &Manifest {
        &self.mir
    }

    pub fn ast(&self) -> &Ast {
        &self.ast
    }
}
