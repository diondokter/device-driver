use std::{fmt::Write, fs, path::Path};

use device_driver_common::identifier::IdentifierType;
use device_driver_diagnostics::{DynError, ResultExt};

use crate::{
    lowering::{PropertyInfo, PropertyName, Shape},
    model::{Block, Buffer, Command, Device, Enum, Extern, Field, FieldSet, Manifest, Register},
};

/// Generate docs for all object shapes
pub fn gen_docs(folder: &Path) -> Result<(), DynError> {
    gen_doc::<Manifest>(folder)?;
    gen_doc::<Device>(folder)?;
    gen_doc::<Block>(folder)?;
    gen_doc::<Register>(folder)?;
    gen_doc::<Command>(folder)?;
    gen_doc::<Buffer>(folder)?;
    gen_doc::<FieldSet>(folder)?;
    gen_doc::<Enum>(folder)?;
    gen_doc::<Extern>(folder)?;
    gen_doc::<Field>(folder)?;

    Ok(())
}

fn gen_doc<S: Shape>(folder: &Path) -> Result<(), DynError> {
    let name = S::NODE_TYPE.to_string();
    let mut doc = String::new();
    let mut shape = S::default();

    let short_properties = S::supported_properties()
        .iter()
        .filter(|p| matches!(p.name, PropertyName::Short(_)))
        .collect::<Vec<_>>();
    let long_properties = S::supported_properties()
        .iter()
        .filter(|p| matches!(p.name, PropertyName::Exact(_) | PropertyName::Any))
        .collect::<Vec<_>>();

    writeln!(doc, "## Shape").into_dyn_result()?;

    writeln!(doc, "> todo: example\n").into_dyn_result()?;

    writeln!(doc, "| Property | Value |").into_dyn_result()?;
    writeln!(doc, "| --- | --- |").into_dyn_result()?;
    writeln!(
        doc,
        "| Identifier namespace | `{:?}` |",
        S::NameIdentifierType::default().runtime_value()
    )
    .into_dyn_result()?;
    writeln!(
        doc,
        "| Supports repeat | `{}` |",
        bool_to_yes_no(shape.repeat().is_some())
    )
    .into_dyn_result()?;
    writeln!(
        doc,
        "| Supports basetype | `{}` |",
        bool_to_yes_no(shape.base_type().is_some())
    )
    .into_dyn_result()?;
    writeln!(
        doc,
        "| Supports conversion type | `{}` |",
        bool_to_yes_no(shape.conversion_type().is_some())
    )
    .into_dyn_result()?;
    writeln!(
        doc,
        "| Supports short properties | {} |",
        if !short_properties.is_empty() {
            "`yes`, see below"
        } else {
            "`no`"
        }
    )
    .into_dyn_result()?;
    writeln!(
        doc,
        "| Supports properties | {} |",
        if !long_properties.is_empty() {
            "`yes`, see below"
        } else {
            "`no`"
        }
    )
    .into_dyn_result()?;
    writeln!(
        doc,
        "| Supports subnodes | {} |",
        if S::supported_subnodes().is_some() {
            "`yes`, see below"
        } else {
            "`no`"
        }
    )
    .into_dyn_result()?;

    if !short_properties.is_empty() {
        writeln!(doc, "## Short properties").into_dyn_result()?;
        writeln!(
            doc,
            "These properties are specified inline in the node definition and are used without name."
        )
        .into_dyn_result()?;
        write_properties(&mut doc, short_properties.as_slice())?;
    }
    if !long_properties.is_empty() {
        writeln!(doc, "## Long properties").into_dyn_result()?;
        writeln!(doc, "These properties are specified in the node body.").into_dyn_result()?;
        write_properties(&mut doc, long_properties.as_slice())?;
    }
    if let Some(subnodes) = S::supported_subnodes() {
        writeln!(doc, "## Possible subnodes").into_dyn_result()?;
        writeln!(
            doc,
            "Subnodes of the following types are allowed in the node body."
        )
        .into_dyn_result()?;
        for subnode in subnodes {
            writeln!(doc, "- [{subnode}]").into_dyn_result()?;
        }
    }

    fs::write(folder.join(name).with_extension("md"), doc)
        .with_message(|| "writing mir shape to file")
}

fn bool_to_yes_no(val: bool) -> &'static str {
    if val { "yes" } else { "no" }
}

fn write_properties<S: Shape>(
    doc: &mut dyn Write,
    properties: &[&PropertyInfo<S>],
) -> Result<(), DynError> {
    for property in properties {
        let name = match property.name {
            PropertyName::Exact(name) => name,
            PropertyName::Any => "*any name*",
            PropertyName::Short(name) => name,
        };

        writeln!(doc, "### {name}").into_dyn_result()?;
        for description_line in property.description.lines() {
            writeln!(doc, "{description_line}").into_dyn_result()?;
        }
        writeln!(doc, "#### Info").into_dyn_result()?;
        writeln!(doc, "- required: `{}`", bool_to_yes_no(property.required)).into_dyn_result()?;
        writeln!(
            doc,
            "- multiple allowed: `{}`",
            bool_to_yes_no(property.multiple_allowed)
        )
        .into_dyn_result()?;
        writeln!(
            doc,
            "- supports doc comments: `{}`",
            bool_to_yes_no(property.supports_doc_comments)
        )
        .into_dyn_result()?;
        writeln!(doc, "#### Allowed expression types").into_dyn_result()?;
        for t in property.allowed_expression_types.iter() {
            writeln!(doc, "- `{}` => `{}`", t, t.get_human_string()).into_dyn_result()?;
        }
    }

    Ok(())
}
