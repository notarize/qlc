use super::graphql::schema::*;
use graphql_parser::query;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum Error {
    UnknownError,
    MissingType(String),
    NotEnumGlobal(String),
    UnknownField(String, String),
    SelectionSetOnWrongType(String),
    MissingTypeCondition,
}

type Result<T> = std::result::Result<T, Error>;

pub struct GlobalTypesCompile {
    pub filename: String,
    pub contents: String,
}

pub struct Compile {
    pub filename: String,
    pub contents: String,
    pub used_global_types: HashSet<String>,
}

fn from_schema_field_scalar(scalar: &ScalarType) -> String {
    match scalar {
        ScalarType::Boolean => String::from("boolean"),
        ScalarType::String | ScalarType::ID => String::from("string"),
        ScalarType::Float | ScalarType::Int => String::from("number"),
        ScalarType::Custom(_) => String::from("any"),
    }
}

fn from_field_description(description: &Option<String>, tab_width: &str) -> String {
    match description {
        Some(desc) => {
            let processed_desc = desc
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>()
                .join(&format!("\n {}* ", tab_width))
                .replace("*/", "");
            format!(
                "/**\n {}* {}\n {}*/\n{}",
                tab_width, processed_desc, tab_width, tab_width
            )
        }
        None => String::from(""),
    }
}

fn from_schema_field_type<F>(
    ctx: &mut CompileContext,
    field_type: &FieldType,
    parent_name: &str,
    field_name: &str,
    mut add_another_type: F,
) -> Result<String>
where
    F: FnMut(&mut CompileContext, &str, &str) -> Result<()>,
{
    let output = match &field_type.definition {
        FieldTypeDefintion::List(sub_field) => {
            let inner_str =
                from_schema_field_type(ctx, &sub_field, parent_name, field_name, add_another_type)?;
            format!("({})[]", inner_str)
        }
        FieldTypeDefintion::Enum(enum_type) => {
            ctx.add_type(enum_type);
            enum_type.to_string()
        }
        FieldTypeDefintion::Object(name) => {
            let object_name = format!("{}_{}", parent_name, field_name);
            add_another_type(ctx, name, &object_name)?;
            object_name
        }
        FieldTypeDefintion::Interface(name) => {
            let object_name = format!("{}_{}", parent_name, field_name);
            add_another_type(ctx, name, &object_name)?;
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
    ctx: &mut CompileContext,
    query_field: &query::Field,
    fields: &HashMap<String, Field>,
    parent_name: &str,
    add_another_type: F,
) -> Result<String>
where
    F: FnMut(&mut CompileContext, &str, &str) -> Result<()>,
{
    let field_name = &query_field.name;
    let user_spec_field_name = query_field
        .alias
        .clone()
        .unwrap_or_else(|| field_name.to_string());
    let field = fields
        .get(field_name)
        .ok_or_else(|| Error::UnknownField(parent_name.to_string(), field_name.clone()))?;
    let field_value = from_schema_field_type(
        ctx,
        &field.type_description,
        parent_name,
        &user_spec_field_name,
        add_another_type,
    )?;
    let doc_comment = from_field_description(&field.description, "  ");
    let prop_line = format!(
        "  {}{}: {};",
        doc_comment, user_spec_field_name, field_value
    );
    Ok(prop_line)
}

fn from_interface_type(
    ctx: &mut CompileContext,
    interface_type: &InterfaceType,
    selection_set: &query::SelectionSet,
    parent_name: &str,
) -> Result<Vec<String>> {
    let mut types = Vec::new();
    let mut concrete_fields = Vec::new();
    let mut spread_implementing_types = HashMap::new();
    // First lets collect all the fields that are implemented right on the interface
    // and also all the types that are spread on this interface.
    for selection in &selection_set.items {
        match selection {
            query::Selection::Field(field_def) => {
                let add_another_type =
                    |ctx: &mut CompileContext, field_type_name: &str, field_object_name: &str| {
                        let mut sub_field_type = from_selection_set(
                            ctx,
                            &field_def.selection_set,
                            field_object_name,
                            field_type_name,
                        )?;
                        types.append(&mut sub_field_type);
                        Ok(())
                    };
                let field = from_field_of_product(
                    ctx,
                    &field_def,
                    &interface_type.fields,
                    parent_name,
                    add_another_type,
                )?;
                concrete_fields.push(field);
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

    // Now we iterate through spread types and add them as top level types
    let mut compiled_interface_types = Vec::with_capacity(spread_implementing_types.len() + 1);
    for (type_name, fragment_def) in spread_implementing_types.iter() {
        let compiled_type_name = format!("{}_{}", parent_name, type_name);
        let mut selection_type = from_selection_set(
            ctx,
            &fragment_def.selection_set,
            &compiled_type_name,
            type_name,
        )?;
        compiled_interface_types.push(compiled_type_name);
        types.append(&mut selection_type);
    }
    let spread_types_rh_def = compiled_interface_types.join(" | ");

    // Now lets compile the interface type itself and add it to the top level types if
    // need be.
    let compiled_interface_type_name = format!("{}_{}", parent_name, interface_type.name);
    let this_interface_type = format!(
        "export interface {} {{\n{}\n}}",
        compiled_interface_type_name,
        concrete_fields.join("\n")
    );
    if !concrete_fields.is_empty() {
        types.push(this_interface_type);
    }

    // Finally we can define this top level type by combining spread types and concrete
    // fields.
    let rh_def = match (
        compiled_interface_types.is_empty(),
        concrete_fields.is_empty(),
    ) {
        (true, true) => return Err(Error::UnknownError),
        (true, false) => compiled_interface_type_name,
        (false, true) => spread_types_rh_def,
        (false, false) => format!(
            "({}) & {}",
            spread_types_rh_def, compiled_interface_type_name
        ),
    };

    let interface = format!("export type {} = {};", parent_name, rh_def);
    types.push(interface);
    Ok(types)
}

fn from_object_type(
    ctx: &mut CompileContext,
    object_type: &ObjectType,
    selection_set: &query::SelectionSet,
    parent_name: &str,
) -> Result<Vec<String>> {
    let mut types = Vec::new();
    let mut fields = Vec::with_capacity(selection_set.items.len());
    for selection in &selection_set.items {
        match selection {
            query::Selection::Field(f_def) => {
                let add_another_type =
                    |ctx: &mut CompileContext, field_type_name: &str, field_object_name: &str| {
                        let mut sub_field_type = from_selection_set(
                            ctx,
                            &f_def.selection_set,
                            field_object_name,
                            field_type_name,
                        )?;
                        types.append(&mut sub_field_type);
                        Ok(())
                    };
                let field = from_field_of_product(
                    ctx,
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
    ctx: &mut CompileContext,
    selection_set: &query::SelectionSet,
    parent_name: &str,
    parent_type_name: &str,
) -> Result<Vec<String>> {
    let parent_type = ctx
        .schema
        .get_type_for_name(parent_type_name)
        .ok_or_else(|| Error::MissingType(parent_type_name.to_string()))?;
    match &parent_type.definition {
        TypeDefintion::Object(object_type) => {
            from_object_type(ctx, object_type, selection_set, parent_name)
        }
        TypeDefintion::Interface(interface_type) => {
            from_interface_type(ctx, interface_type, selection_set, parent_name)
        }
        TypeDefintion::Scalar(_) => {
            Err(Error::SelectionSetOnWrongType(parent_type_name.to_string()))
        }
        TypeDefintion::Enum(_) => Err(Error::SelectionSetOnWrongType(parent_type_name.to_string())),
    }
}

fn from_query(ctx: &mut CompileContext, query: &query::Query) -> Result<(String, String)> {
    let query_name = "Query";
    let name = query.name.clone().unwrap_or_else(|| query_name.to_string());
    let type_defs = from_selection_set(ctx, &query.selection_set, &name, query_name)?;
    let imports = ctx.compile_imports();
    let contents = format!("{}{}", imports, type_defs.join("\n\n"));
    Ok((name, contents))
}

fn from_fragment(
    ctx: &mut CompileContext,
    fragment: &query::FragmentDefinition,
) -> Result<(String, String)> {
    let name = fragment.name.clone();
    let query::TypeCondition::On(type_name) = &fragment.type_condition;
    let type_defs = from_selection_set(ctx, &fragment.selection_set, &name, type_name)?;
    let imports = ctx.compile_imports();
    let contents = format!("{}{}", imports, type_defs.join("\n\n"));
    Ok((name, contents))
}

fn from_operation(
    ctx: &mut CompileContext,
    operation: &query::OperationDefinition,
) -> Result<(String, String)> {
    match operation {
        query::OperationDefinition::Query(query) => from_query(ctx, query),
        _ => Err(Error::UnknownError),
    }
}

pub fn compile(definition: &query::Definition, schema: &Schema) -> Result<Compile> {
    let mut ctx = CompileContext::new(schema);
    let (name, contents) = match definition {
        query::Definition::Operation(op_def) => from_operation(&mut ctx, op_def),
        query::Definition::Fragment(frag_def) => from_fragment(&mut ctx, frag_def),
    }?;
    let filename = format!("{}.ts", name);
    Ok(Compile {
        filename,
        contents,
        used_global_types: ctx.global_types,
    })
}

fn enum_def_from_type(name: &str, description: &Option<String>, enum_type: &EnumType) -> String {
    let doc_comment = from_field_description(description, "");
    let values = enum_type
        .possible_values
        .iter()
        .map(|value| format!("  {} = \"{}\",", value, value))
        .collect::<Vec<String>>()
        .join("\n");
    format!("{}export enum {} {{\n{}\n}}", doc_comment, name, values)
}

fn enums_from_names(schema: &Schema, global_names: &HashSet<String>) -> Result<Vec<String>> {
    let mut enums = Vec::new();
    for name in global_names {
        let global_type = schema
            .get_type_for_name(name)
            .ok_or_else(|| Error::MissingType(name.to_string()))?;
        match &global_type.definition {
            TypeDefintion::Enum(enum_type) => {
                enums.push(enum_def_from_type(
                    name,
                    &global_type.description,
                    enum_type,
                ));
            }
            _ => return Err(Error::NotEnumGlobal(name.to_string())),
        }
    }
    Ok(enums)
}

pub fn compile_globals(
    schema: &Schema,
    global_names: &HashSet<String>,
) -> Result<GlobalTypesCompile> {
    let enums = enums_from_names(schema, global_names)?;
    Ok(GlobalTypesCompile {
        filename: String::from("globalTypes.ts"),
        contents: enums.join("\n\n"),
    })
}

struct CompileContext<'a> {
    schema: &'a Schema,
    global_types: HashSet<String>,
}

impl<'a> CompileContext<'a> {
    fn new(schema: &'a Schema) -> Self {
        CompileContext {
            schema,
            global_types: HashSet::new(),
        }
    }

    fn compile_imports(&self) -> String {
        if self.global_types.is_empty() {
            return String::from("");
        }
        let names = self
            .global_types
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<String>>();
        format!(
            "import {{ {} }} from \"__generated__/globalTypes\";\n\n",
            names.join(", ")
        )
    }

    fn add_type(&mut self, name: &str) {
        self.global_types.insert(name.to_string());
    }
}
