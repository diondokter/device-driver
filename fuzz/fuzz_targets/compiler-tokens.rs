#![no_main]

use device_driver_codegen::Target;
use device_driver_diagnostics::{Diagnostics, ResultExt};
use device_driver_lexer::Token;
use libfuzzer_sys::fuzz_target;

// Goal: Don't have panics or ICE's
fuzz_target!(|tokens: Vec<Token>| {
    let target = Target::Rust;
    let mut diagnostics = Diagnostics::new();

    let mut source = String::new();
    Token::formatted_print(&mut source, tokens.iter()).unwrap();

    let tokens = device_driver_lexer::lex(&source);
    let ast = device_driver_parser::parse(&tokens, &mut diagnostics);
    let mir = device_driver_mir::lower_ast(ast, &mut diagnostics)
        .with_message(|| "could not lower AST to MIR")
        .unwrap();
    let lir = device_driver_lir::lower_mir(mir)
        .with_message(|| "could not lower MIR to LIR")
        .unwrap();
    let _ = device_driver_codegen::codegen(target, &lir, &source, &target.get_compile_options());
});
