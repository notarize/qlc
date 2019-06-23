use super::schema::{Field, FieldType, Schema, Type, ScalarType};
use graphql_parser::query;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    MissingType(String),
    UnknownField(String),
    InvariantViolation(String),
    UnknownError,
}

#[derive(Debug)]
pub enum NamedType {
    UserDefined,
    Null,
    Boolean,
    String,
    Float,
    Int,
    ID,
    Unknown,
    List(Vec<NamedType>),
}

#[derive(Debug)]
pub struct SumTypeDefinition {
    pub names: Vec<NamedType>,
}

#[derive(Debug)]
pub struct ProductTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug)]
pub enum TypeDefinition {
    Product(ProductTypeDefinition),
    Sum(SumTypeDefinition),
}

#[derive(Debug)]
pub struct FieldDefinition {
    pub name: String,
    pub description: Option<String>,
    pub field_type: SumTypeDefinition,
}

fn make_named_type_from_scalar(scalar: &ScalarType) -> NamedType {
    match scalar {
        ScalarType::Custom(_) => NamedType::Unknown,
        ScalarType::Boolean => NamedType::Boolean,
        ScalarType::String => NamedType::String,
        ScalarType::Float => NamedType::Float,
        ScalarType::Int => NamedType::Int,
        ScalarType::ID => NamedType::ID,
    }
}

fn make_field_sum_type(field_type_description: &FieldType) -> SumTypeDefinition {
    match field_type_description {
        FieldType::List(md, sub_field) => {
            let inner_type_names = make_field_sum_type(sub_field).names;
            let mut names = vec![NamedType::List(inner_type_names)];
            if md.nullable {
                names.push(NamedType::Null);
            }
            SumTypeDefinition { names }
        }
        FieldType::Enum(md, enum_type) => {
            let mut names = vec![NamedType::UserDefined];
            if md.nullable {
                names.push(NamedType::Null);
            }
            SumTypeDefinition { names }
        }
        FieldType::Object(md, name) => {
            let mut names = vec![NamedType::UserDefined];
            if md.nullable {
                names.push(NamedType::Null);
            }
            SumTypeDefinition { names }
        }
        FieldType::Scalar(md, sc_type) => {
            let mut names = vec![make_named_type_from_scalar(sc_type)];
            if md.nullable {
                names.push(NamedType::Null);
            }
            SumTypeDefinition { names }
        }
    }
}

fn make_field_def(
    parsed: &query::Field,
    fields: &HashMap<String, Field>,
) -> Result<FieldDefinition, Error> {
    let name = parsed.name.clone();
    let field = fields
        .get(&name)
        .ok_or_else(|| Error::UnknownField(parsed.name.clone()))?;
    Ok(FieldDefinition {
        name: parsed.alias.clone().unwrap_or(name),
        description: field.description.clone(),
        field_type: make_field_sum_type(&field.type_description),
    })
}

fn make_query_defs(parsed: query::Query, schema: &Schema) -> Result<(String, Vec<TypeDefinition>), Error> {
    let query_name = String::from("Query");
    let query_type = match schema.get_type_for_name(&query_name) {
        Some(t) => t,
        None => return Err(Error::MissingType(query_name)),
    };
    let query_object_type = match query_type {
        Type::Object(_, object_type) => object_type,
        _ => {
            return Err(Error::InvariantViolation(String::from(
                "Query type cannot be scalar",
            )))
        }
    };
    let name = parsed.name.unwrap_or_else(|| String::from("Query"));
    let mut fields = Vec::with_capacity(parsed.selection_set.items.len());
    for selection in parsed.selection_set.items {
        match selection {
            query::Selection::Field(f_def) => {
                let field = make_field_def(&f_def, &query_object_type.fields)?;
                fields.push(field);
            }
            _ => return Err(Error::UnknownError),
        }
    }
    Ok((
        name.clone(),
        vec![TypeDefinition::Product(ProductTypeDefinition { name, fields })],
    ))
}

#[derive(Debug)]
pub struct OperationDefinition {
    pub name: String,
    pub type_defs: Vec<TypeDefinition>,
}

fn make_op_def(
    parsed: query::OperationDefinition,
    schema: &Schema,
) -> Result<OperationDefinition, Error> {
    let (name, type_defs) = match parsed {
        query::OperationDefinition::Query(query) => make_query_defs(query, schema),
        _ => return Err(Error::UnknownError),
    }?;
    Ok(OperationDefinition { name, type_defs })
}

#[derive(Debug)]
pub struct DocumentDefinition {
    pub definitions: Vec<OperationDefinition>,
}

pub fn make_document_defs(
    parsed: query::Document,
    schema: &Schema,
) -> Result<DocumentDefinition, Error> {
    let mut op_defs = Vec::with_capacity(parsed.definitions.len());
    for definition in parsed.definitions {
        let op_attempt = match definition {
            query::Definition::Operation(op) => make_op_def(op, schema),
            _ => return Err(Error::UnknownError),
        };
        op_defs.push(op_attempt?);
    }
    Ok(DocumentDefinition {
        definitions: op_defs,
    })
}
