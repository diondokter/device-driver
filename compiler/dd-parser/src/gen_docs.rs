use std::path::Path;

use chumsky::Parser;
use device_driver_diagnostics::{DynError, ResultExt};

use crate::{InputType, RichExtra};

pub fn gen_docs(output_path: &Path) -> Result<(), DynError> {
    gen_railroad(&output_path.join("node.svg"), super::node())
        .with_message(|| "generating node railroad diagram")?;
    gen_railroad(&output_path.join("range.svg"), super::range())
        .with_message(|| "generating node railroad diagram")?;
    gen_railroad(&output_path.join("byte-array.svg"), super::byte_array())
        .with_message(|| "generating node railroad diagram")?;
    gen_railroad(
        &output_path.join("simple-expression.svg"),
        super::simple_expression(),
    )
    .with_message(|| "generating node railroad diagram")?;
    gen_railroad(&output_path.join("repeat.svg"), super::repeat())
        .with_message(|| "generating node railroad diagram")?;
    Ok(())
}

fn gen_railroad<'src: 'tokens, 'tokens, O>(
    output_path: &Path,
    parser: impl Parser<'tokens, InputType<'tokens, 'src>, O, RichExtra<'tokens, 'src>>,
) -> Result<(), DynError> {
    std::fs::write(output_path, parser.debug().to_railroad_svg().to_string())
        .with_message(|| format!("writing diagram to {}", output_path.display()))?;

    Ok(())
}
