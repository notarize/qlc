use super::complex::FieldIR;
use super::{CompileContext, Error, Parent, Result, Typescript};
use crate::graphql::schema::{Documentation, FieldType, FieldTypeDefinition, ScalarType};
use std::collections::HashSet;

#[derive(Debug)]
pub struct TypeNameDescription {
    pub aliases: HashSet<String>,
    pub opt_possible_types: Option<Vec<String>>,
}

pub type TSFields<'a> = Vec<(String, Option<&'a FieldIR>, Typescript)>;

fn compile_field_type(
    ctx: &mut CompileContext,
    field_type: &FieldType,
    parent: &Parent,
    field_name: &str,
    user_specified_name: &str,
) -> Result<(String, String)> {
    let (name, output) = match &field_type.definition {
        FieldTypeDefinition::List(sub_field_type) => {
            let (inner_name, inner_str) = compile_field_type(
                ctx,
                &sub_field_type,
                parent,
                field_name,
                user_specified_name,
            )?;
            (inner_name, format!("({})[]", inner_str))
        }
        FieldTypeDefinition::Enum(enum_type) => {
            ctx.add_type(enum_type);
            (enum_type.to_string(), enum_type.to_string())
        }
        FieldTypeDefinition::Object(_)
        | FieldTypeDefinition::Interface(_)
        | FieldTypeDefinition::Union(_) => {
            let output = format!("{}_{}", parent.compiled_name, user_specified_name);
            (output.clone(), output)
        }
        FieldTypeDefinition::Scalar(sc_type) => {
            if field_name == "__typename" {
                let type_literal = format!("\"{}\"", parent.type_name);
                (type_literal.clone(), type_literal)
            } else {
                (compile_scalar(&sc_type), compile_scalar(&sc_type))
            }
        }
        FieldTypeDefinition::InputObject(name) => {
            return Err(Error::InputObjectInOutput(name.to_string()))
        }
    };
    if field_type.nullable {
        return Ok((name, format!("{} | null", output)));
    }
    Ok((name, output))
}

pub fn compile_documentation(docs: &Documentation, tab_width: usize) -> String {
    match docs {
        Some(docs) => {
            let tab = " ".repeat(tab_width);
            let processed_desc = docs
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>()
                .join(&format!("\n {}* ", tab))
                .replace("*/", "");
            format!("/**\n {}* {}\n {}*/\n{}", tab, processed_desc, tab, tab,)
        }
        None => String::from(""),
    }
}

pub fn compile_ts_fields<'a>(
    ctx: &mut CompileContext,
    parent: &Parent,
    collection: &[&'a FieldIR],
    typename_description: Option<&TypeNameDescription>,
) -> Result<TSFields<'a>> {
    let mut sorted_collection = collection.iter().collect::<Vec<_>>();
    sorted_collection.sort_unstable_by_key(|field_ir| &field_ir.user_specified_name);
    let has_typename_desc = typename_description.is_some();
    let type_name_fields = typename_description.into_iter().flat_map(|type_desc| {
        let type_literal = type_desc
            .opt_possible_types
            .as_ref()
            .map(|types| {
                let names = types
                    .iter()
                    .map(|t| format!("\"{}\"", t))
                    .collect::<Vec<_>>();
                names.join(" | ")
            })
            .unwrap_or_else(|| format!("\"{}\"", parent.type_name));
        let mut sorted_alias = type_desc.aliases.iter().collect::<Vec<_>>();
        sorted_alias.sort_unstable();
        sorted_alias.into_iter().map(move |alias| {
            Ok((
                "__typename".to_string(),
                None,
                format!("  {}: {};", alias, type_literal),
            ))
        })
    });
    let reg_fields = sorted_collection
        .into_iter()
        .filter(|field_ir| !has_typename_desc || field_ir.field.name != "__typename")
        .map(|field_ir| {
            let user_specified_name = &field_ir.user_specified_name;
            let schema_field = &field_ir.field;
            let doc_comment = compile_documentation(&schema_field.documentation, 2);
            let (field_type_name, compiled_value) = compile_field_type(
                ctx,
                &schema_field.type_description,
                parent,
                &schema_field.name,
                user_specified_name,
            )?;
            Ok((
                field_type_name,
                Some(*field_ir),
                format!(
                    "  {}{}: {};",
                    doc_comment, user_specified_name, compiled_value
                ),
            ))
        });
    let ts_fields = type_name_fields.chain(reg_fields).collect::<Result<_>>()?;
    Ok(ts_fields)
}

pub fn compile_scalar(scalar: &ScalarType) -> String {
    match scalar {
        ScalarType::Boolean => String::from("boolean"),
        ScalarType::String | ScalarType::ID => String::from("string"),
        ScalarType::Float | ScalarType::Int => String::from("number"),
        ScalarType::Custom(_) => String::from("any"),
    }
}
