use super::ParsedTextType;
use crate::cli::PrintableMessage;
use crate::graphql::ir::CompileContext;
use crate::graphql::schema::field as schema_field;
use graphql_parser::query as parsed_query;
use graphql_parser::Pos;
use std::convert::TryFrom;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    UnknownCustomeVariableType(String, Pos),
    UnprocessableVariableType(String, Pos),
    ListOfListNotSupported(String, Pos),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum VariableType<'a> {
    Id,
    String,
    Float,
    Int,
    Boolean,
    Custom(&'a str),
}

impl<'a, 'b, 'c> TryFrom<(&'a CompileContext<'a, 'b>, &'c str, Pos)> for VariableType<'c> {
    type Error = Error;
    fn try_from(from: (&'a CompileContext<'a, 'b>, &'c str, Pos)) -> Result<Self> {
        let var = match from.1 {
            "ID" => VariableType::Id,
            "String" => VariableType::String,
            "Float" => VariableType::Float,
            "Int" => VariableType::Int,
            "Boolean" => VariableType::Boolean,
            custom_type_name => match from.0.schema.get_type_for_name(custom_type_name) {
                None => {
                    return Err(Error::UnknownCustomeVariableType(
                        custom_type_name.to_string(),
                        from.2,
                    ))
                }
                Some(_) => VariableType::Custom(custom_type_name),
            },
        };
        Ok(var)
    }
}

impl From<(&str, &Path, Error)> for PrintableMessage {
    fn from((contents, file_path, error): (&str, &Path, Error)) -> Self {
        match error {
            Error::UnknownCustomeVariableType(name, position) => {
                PrintableMessage::new_compile_error(
                    &format!("unknown type for variable `{}`", name),
                    file_path,
                    contents,
                    &position,
                    None,
                )
            }
            Error::UnprocessableVariableType(name, position) => {
                PrintableMessage::new_program_error(
                    &format!("failed to process variable `{}`", name),
                    file_path,
                    contents,
                    &position,
                    Some(
                        "This is most likely a bug in QLC or some unsupported variable type. Please report a bug!"
                    ),
                )
            }
            Error::ListOfListNotSupported(name, position) => PrintableMessage::new_compile_error(
                &format!("unsupported list of lists type for variable `{}`", name),
                file_path,
                contents,
                &position,
                Some(
                    "QLC does not support lists of lists as variable types"
                ),
            ),
        }
    }
}

#[derive(Debug)]
pub struct Variable<'a> {
    pub prop_name: String,
    pub type_modifier: schema_field::FieldTypeModifier,
    pub type_ir: VariableType<'a>,
}

pub fn try_build_variable_ir<'a, 'b, 'c>(
    context: &'b CompileContext<'b, 'c>,
    defs: &'a [parsed_query::VariableDefinition<'_, ParsedTextType>],
) -> Result<Option<Vec<Variable<'a>>>> {
    if defs.is_empty() {
        return Ok(None);
    }
    defs.iter()
        .map(|def| {
            let (type_modifier, graph_name) = unwrap_var_def(def)?;
            Ok(Variable {
                prop_name: def.name.clone(),
                type_modifier,
                type_ir: TryFrom::try_from((context, graph_name, def.position))?,
            })
        })
        .collect::<Result<Vec<Variable<'_>>>>()
        .map(Some)
}

fn unwrap_var_def<'a, 'b>(
    var_def: &'a parsed_query::VariableDefinition<'b, ParsedTextType>,
) -> Result<(schema_field::FieldTypeModifier, &'a str)> {
    let (type_mod, name) = match &var_def.var_type {
        parsed_query::Type::NamedType(name) => (schema_field::FieldTypeModifier::Nullable, name),
        parsed_query::Type::NonNullType(inner_type) => match inner_type.as_ref() {
            parsed_query::Type::NamedType(name) => (schema_field::FieldTypeModifier::None, name),
            parsed_query::Type::NonNullType(_) => {
                return Err(Error::UnprocessableVariableType(
                    var_def.name.to_string(),
                    var_def.position,
                ))
            }
            parsed_query::Type::ListType(inner_type) => match inner_type.as_ref() {
                parsed_query::Type::NamedType(name) => {
                    (schema_field::FieldTypeModifier::ListOfNullable, name)
                }
                parsed_query::Type::NonNullType(inner_type) => match inner_type.as_ref() {
                    parsed_query::Type::NamedType(name) => {
                        (schema_field::FieldTypeModifier::List, name)
                    }
                    _ => {
                        return Err(Error::UnprocessableVariableType(
                            var_def.name.to_string(),
                            var_def.position,
                        ))
                    }
                },
                parsed_query::Type::ListType(_) => {
                    return Err(Error::ListOfListNotSupported(
                        var_def.name.to_string(),
                        var_def.position,
                    ))
                }
            },
        },
        parsed_query::Type::ListType(inner_list) => match inner_list.as_ref() {
            parsed_query::Type::ListType(_) => {
                return Err(Error::ListOfListNotSupported(
                    var_def.name.to_string(),
                    var_def.position,
                ))
            }
            parsed_query::Type::NamedType(name) => (
                schema_field::FieldTypeModifier::NullableListOfNullable,
                name,
            ),
            parsed_query::Type::NonNullType(inner_type) => match inner_type.as_ref() {
                parsed_query::Type::NamedType(name) => {
                    (schema_field::FieldTypeModifier::NullableList, name)
                }
                _ => {
                    return Err(Error::UnprocessableVariableType(
                        var_def.name.to_string(),
                        var_def.position,
                    ))
                }
            },
        },
    };
    Ok((type_mod, name))
}
