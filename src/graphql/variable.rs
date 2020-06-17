use crate::graphql::schema::field as schema_field;
use graphql_parser::query as parsed_query;

#[derive(Debug)]
pub enum Error {
    UnparseableInputType,
    ListOfListNotSupported,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum VariableType<'a> {
    ID,
    String,
    Float,
    Int,
    Boolean,
    Custom(&'a str),
}

impl<'a> From<&'a str> for VariableType<'a> {
    fn from(type_name: &'a str) -> Self {
        match type_name {
            "ID" => VariableType::ID,
            "String" => VariableType::String,
            "Float" => VariableType::Float,
            "Int" => VariableType::Int,
            "Boolean" => VariableType::Boolean,
            x => VariableType::Custom(x),
        }
    }
}

#[derive(Debug)]
pub struct Variable<'a> {
    pub prop_name: String,
    pub type_modifier: schema_field::FieldTypeModifier,
    pub type_ir: VariableType<'a>,
}

pub fn try_build_variable_ir<'a>(
    defs: &'a [parsed_query::VariableDefinition],
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
                type_ir: graph_name.into(),
            })
        })
        .collect::<Result<Vec<Variable<'_>>>>()
        .map(Some)
}

fn unwrap_var_def(
    var_def: &parsed_query::VariableDefinition,
) -> Result<(schema_field::FieldTypeModifier, &str)> {
    let (type_mod, name) = match &var_def.var_type {
        parsed_query::Type::NamedType(name) => (schema_field::FieldTypeModifier::Nullable, name),
        parsed_query::Type::NonNullType(inner_type) => match inner_type.as_ref() {
            parsed_query::Type::NamedType(name) => (schema_field::FieldTypeModifier::None, name),
            parsed_query::Type::NonNullType(_) => return Err(Error::UnparseableInputType),
            parsed_query::Type::ListType(inner_type) => match inner_type.as_ref() {
                parsed_query::Type::NamedType(name) => {
                    (schema_field::FieldTypeModifier::ListOfNullable, name)
                }
                parsed_query::Type::NonNullType(inner_type) => match inner_type.as_ref() {
                    parsed_query::Type::NamedType(name) => {
                        (schema_field::FieldTypeModifier::List, name)
                    }
                    _ => return Err(Error::UnparseableInputType),
                },
                parsed_query::Type::ListType(_) => return Err(Error::ListOfListNotSupported),
            },
        },
        parsed_query::Type::ListType(inner_list) => match inner_list.as_ref() {
            parsed_query::Type::ListType(_) => return Err(Error::ListOfListNotSupported),
            parsed_query::Type::NamedType(name) => (
                schema_field::FieldTypeModifier::NullableListOfNullable,
                name,
            ),
            parsed_query::Type::NonNullType(inner_type) => match inner_type.as_ref() {
                parsed_query::Type::NamedType(name) => {
                    (schema_field::FieldTypeModifier::NullableList, name)
                }
                _ => return Err(Error::UnparseableInputType),
            },
        },
    };
    Ok((type_mod, name))
}
