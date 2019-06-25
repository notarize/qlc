use super::graphql::schema::*;
use graphql_parser::query;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    UnknownError,
    MissingType(String),
    UnknownField(String),
    SelectionSetOnScalar(String),
    MissingTypeCondition,
}

type Result<T> = std::result::Result<T, Error>;

fn from_schema_field_scalar(scalar: &ScalarType) -> String {
    match scalar {
        ScalarType::Null => String::from("null"),
        ScalarType::Boolean => String::from("boolean"),
        ScalarType::String | ScalarType::ID => String::from("string"),
        ScalarType::Float | ScalarType::Int => String::from("number"),
        ScalarType::Custom(_) => String::from("any"),
    }
}

fn from_field_description(description: &Option<String>) -> String {
    match description {
        Some(desc) => {
            let processed_desc = desc
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>()
                .join("\n   * ")
                .replace("*/", "");
            format!("/**\n   * {}\n   */\n  ", processed_desc)
        }
        None => String::from(""),
    }
}

fn from_schema_field_type<F>(
    field_type: &FieldType,
    parent_name: &str,
    field_name: &str,
    mut add_another_type: F,
) -> Result<String>
where
    F: FnMut(&str, &str) -> Result<()>,
{
    let output = match &field_type.definition {
        FieldTypeDefintion::List(sub_field) => {
            let inner_str =
                from_schema_field_type(&sub_field, parent_name, field_name, add_another_type)?;
            format!("({})[]", inner_str)
        }
        FieldTypeDefintion::Enum(enum_type) => {
            return Err(Error::UnknownError);
        }
        FieldTypeDefintion::Object(name) => {
            let object_name = format!("{}_{}", parent_name, field_name);
            add_another_type(name, &object_name)?;
            object_name
        }
        FieldTypeDefintion::Interface(name) => {
            let object_name = format!("{}_{}", parent_name, field_name);
            add_another_type(name, &object_name)?;
            object_name
        }
        FieldTypeDefintion::Scalar(sc_type) => from_schema_field_scalar(&sc_type),
    };
    if field_type.nullable {
        return Ok(format!("{} | null", output));
    }
    Ok(output)
}

fn from_field_of_product<F>(
    query_field: &query::Field,
    fields: &HashMap<String, Field>,
    parent_name: &str,
    add_another_type: F,
) -> Result<String>
where
    F: FnMut(&str, &str) -> Result<()>,
{
    let field_name = &query_field.name;
    let user_spec_field_name = query_field
        .alias
        .clone()
        .unwrap_or_else(|| field_name.to_string());
    let field = fields
        .get(field_name)
        .ok_or_else(|| Error::UnknownField(query_field.name.clone()))?;
    let field_value = from_schema_field_type(
        &field.type_description,
        parent_name,
        &user_spec_field_name,
        add_another_type,
    )?;
    let doc_comment = from_field_description(&field.description);
    let prop_line = format!(
        "  {}{}: {};",
        doc_comment, user_spec_field_name, field_value
    );
    Ok(prop_line)
}

fn from_interface_type(
    interface_type: &InterfaceType,
    selection_set: &query::SelectionSet,
    schema: &Schema,
    parent_name: &str,
) -> Result<Vec<String>> {
    let mut types = Vec::new();
    let mut this_interface_fields = Vec::new();
    let mut spread_implementing_types = HashMap::new();
    for selection in &selection_set.items {
        match selection {
            query::Selection::Field(field_def) => {
                let add_another_type = |field_type_name: &str, field_object_name: &str| {
                    let mut sub_field_type = from_selection_set(
                        &field_def.selection_set,
                        schema,
                        field_object_name,
                        field_type_name,
                    )?;
                    types.append(&mut sub_field_type);
                    Ok(())
                };
                let field = from_field_of_product(
                    &field_def,
                    &interface_type.fields,
                    parent_name,
                    add_another_type,
                )?;
                this_interface_fields.push(field);
            }
            query::Selection::InlineFragment(fragment_def) => {
                let type_name = match &fragment_def.type_condition {
                    Some(query::TypeCondition::On(name)) => name,
                    _ => return Err(Error::MissingTypeCondition),
                };
                spread_implementing_types.insert(type_name, fragment_def);
            }
            _ => return Err(Error::UnknownError),
        }
    }
    let mut this_interface_types = Vec::with_capacity(spread_implementing_types.len() + 1);
    for (type_name, fragment_def) in spread_implementing_types.iter() {
        let compiled_type_name = format!("{}_{}", parent_name, type_name);
        let mut selection_type = from_selection_set(
            &fragment_def.selection_set,
            schema,
            &compiled_type_name,
            type_name,
        )?;
        this_interface_types.push(compiled_type_name);
        types.append(&mut selection_type);
    }
    let mut main_rh_def = this_interface_types.join(" | ");
    if !this_interface_fields.is_empty() {
        let compiled_type_name = format!("{}_{}", parent_name, interface_type.name);
        let this_interface_type = format!(
            "export interface {} {{\n{}\n}}",
            compiled_type_name,
            this_interface_fields.join("\n")
        );
        main_rh_def = format!("({}) & {}", main_rh_def, compiled_type_name);
        types.push(this_interface_type);
    }
    let interface = format!("export type {} = {};", parent_name, main_rh_def,);
    types.push(interface);
    Ok(types)
}

fn from_object_type(
    object_type: &ObjectType,
    selection_set: &query::SelectionSet,
    schema: &Schema,
    parent_name: &str,
) -> Result<Vec<String>> {
    let mut types = Vec::new();
    let mut fields = Vec::with_capacity(selection_set.items.len());
    for selection in &selection_set.items {
        match selection {
            query::Selection::Field(f_def) => {
                let add_another_type = |field_type_name: &str, field_object_name: &str| {
                    let mut sub_field_type = from_selection_set(
                        &f_def.selection_set,
                        schema,
                        field_object_name,
                        field_type_name,
                    )?;
                    types.append(&mut sub_field_type);
                    Ok(())
                };
                let field = from_field_of_product(
                    &f_def,
                    &object_type.fields,
                    parent_name,
                    add_another_type,
                )?;
                fields.push(field);
            }
            _ => return Err(Error::UnknownError),
        }
    }
    let interface = format!(
        "export interface {} {{\n{}\n}}",
        parent_name,
        fields.join("\n")
    );
    types.push(interface);
    Ok(types)
}

fn from_selection_set(
    selection_set: &query::SelectionSet,
    schema: &Schema,
    parent_name: &str,
    parent_type_name: &str,
) -> Result<Vec<String>> {
    let parent_type = schema
        .get_type_for_name(parent_type_name)
        .ok_or_else(|| Error::MissingType(parent_type_name.to_string()))?;
    match &parent_type.definition {
        TypeDefintion::Object(object_type) => {
            from_object_type(object_type, selection_set, schema, parent_name)
        }
        TypeDefintion::Interface(interface_type) => {
            from_interface_type(interface_type, selection_set, schema, parent_name)
        }
        TypeDefintion::Scalar(name) => Err(Error::SelectionSetOnScalar(name.clone())),
        TypeDefintion::Enum(enum_type) => Err(Error::UnknownError),
    }
}

fn from_query(query: &query::Query, schema: &Schema) -> Result<(String, String)> {
    let query_name = "Query";
    let name = query.name.clone().unwrap_or_else(|| query_name.to_string());
    let type_defs = from_selection_set(&query.selection_set, schema, &name, query_name)?;
    Ok((name, type_defs.join("\n\n")))
}

fn from_operation(
    operation: &query::OperationDefinition,
    schema: &Schema,
) -> Result<(String, String)> {
    match operation {
        query::OperationDefinition::Query(query) => from_query(query, schema),
        _ => Err(Error::UnknownError),
    }
}

pub fn compile(definition: &query::Definition, schema: &Schema) -> Result<(String, String)> {
    let (name, compiled_contents) = match definition {
        query::Definition::Operation(op_def) => from_operation(op_def, schema),
        query::Definition::Fragment(frag_def) => return Err(Error::UnknownError),
    }?;
    let filename = format!("{}.ts", name);
    Ok((filename, compiled_contents))
}
