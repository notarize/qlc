use super::graphql::schema::*;
use field::{compile_documentation, compile_scalar};
use graphql_parser::query;
use std::collections::{HashMap, HashSet};

mod complex;
mod field;
mod fragment;
mod operation;
mod variable;

const HEADER: &str = "/* tslint:disable */
/* eslint-disable */
// This file was automatically generated and should not be edited.

";

#[derive(Debug)]
pub enum Error {
    UnknownError,
    UnionOnlyAllowedTypename(String),
    UnionCanOnlyHaveObjectImplementors,
    MissingType(String),
    NotGlobalType(String),
    UnknownField(String, String),
    UnknownFragment(String),
    OperationUnsupported,
    SelectionSetOnWrongType(String),
    MissingTypeCondition,
    InvalidFieldDef(String),
    InputObjectInOutput(String),
    OutputInInput(String),
}

type Result<T> = std::result::Result<T, Error>;

type Typescript = String;

pub struct GlobalTypesCompile {
    pub filename: String,
    pub contents: String,
}

pub struct Compile {
    pub filename: String,
    pub contents: String,
    pub used_global_types: HashSet<String>,
}

#[derive(Debug)]
pub struct Parent {
    pub compiled_name: String,
    pub type_name: String,
}

pub fn compile(
    definition: &query::Definition,
    schema: &Schema,
    imported_fragments: HashMap<String, query::FragmentDefinition>,
) -> Result<Compile> {
    let mut ctx = CompileContext::new(schema, imported_fragments);
    let (name, contents) = match definition {
        query::Definition::Operation(op_def) => operation::from_operation(&mut ctx, op_def),
        query::Definition::Fragment(frag_def) => fragment::from_fragment(&mut ctx, frag_def),
    }?;
    let filename = format!("{}.ts", name);
    let contents = format!("{}{}", HEADER, contents);
    Ok(Compile {
        filename,
        contents,
        used_global_types: ctx.global_types,
    })
}

fn from_input_def_field_def(field_name: &str, field_type: &FieldType) -> Result<String> {
    let output = match &field_type.definition {
        FieldTypeDefinition::List(sub_field) => {
            let inner_str = from_input_def_field_def(field_name, &sub_field)?;
            format!("({})[]", inner_str)
        }
        FieldTypeDefinition::Scalar(sc_type) => compile_scalar(&sc_type),
        FieldTypeDefinition::Enum(enum_type) => enum_type.to_string(),
        FieldTypeDefinition::InputObject(name) => name.to_string(),
        _ => return Err(Error::InvalidFieldDef(field_name.to_string())),
    };
    if field_type.nullable {
        return Ok(format!("{} | null", output));
    }
    Ok(output)
}

fn input_def_from_type(input_type: &InputObjectType) -> Result<String> {
    let mut fields = Vec::new();
    let mut sorted = input_type.fields.iter().collect::<Vec<(&String, &Field)>>();
    sorted.sort_unstable_by_key(|item| item.0);
    for (name, field) in sorted {
        let doc = compile_documentation(&field.documentation, 2);
        let field_type = from_input_def_field_def(name, &field.type_description)?;
        let ts_field = if field.type_description.nullable {
            format!("  {}{}?: {};", doc, name, field_type)
        } else {
            format!("  {}{}: {};", doc, name, field_type)
        };
        fields.push(ts_field);
    }
    Ok(format!(
        "export interface {} {{\n{}\n}}",
        input_type.name,
        fields.join("\n")
    ))
}

fn enum_def_from_type(name: &str, documentation: &Documentation, enum_type: &EnumType) -> String {
    let doc_comment = compile_documentation(documentation, 0);
    let values = enum_type
        .possible_values
        .iter()
        .map(|value| format!("  {} = \"{}\",", value, value))
        .collect::<Vec<String>>()
        .join("\n");
    format!("{}export enum {} {{\n{}\n}}", doc_comment, name, values)
}

fn add_sub_input_objects<'a>(
    name_to_type: &mut HashMap<&'a str, &'a Type>,
    schema: &'a Schema,
    current_type: &'a Type,
) -> Result<()> {
    if let TypeDefinition::InputObject(input_object_type) = &current_type.definition {
        for field in input_object_type.fields.values() {
            add_sub_input_object_field(name_to_type, schema, &field.type_description, &field.name)?;
        }
    }
    Ok(())
}

fn add_sub_input_object_field<'a>(
    name_to_type: &mut HashMap<&'a str, &'a Type>,
    schema: &'a Schema,
    field_type_description: &'a FieldType,
    field_name: &'a str,
) -> Result<()> {
    match &field_type_description.definition {
        FieldTypeDefinition::InputObject(input_obj_name) => {
            let global_type = schema
                .get_type_for_name(input_obj_name)
                .ok_or_else(|| Error::MissingType(input_obj_name.to_string()))?;
            name_to_type.insert(input_obj_name, global_type);
            add_sub_input_objects(name_to_type, schema, global_type)?;
        }
        FieldTypeDefinition::Enum(enum_name) => {
            let global_type = schema
                .get_type_for_name(enum_name)
                .ok_or_else(|| Error::MissingType(enum_name.to_string()))?;
            name_to_type.insert(enum_name, global_type);
        }
        FieldTypeDefinition::List(inner) => {
            add_sub_input_object_field(name_to_type, schema, inner, field_name)?;
        }
        FieldTypeDefinition::Scalar(_) => {}
        _ => return Err(Error::OutputInInput(field_name.to_string())),
    }
    Ok(())
}

fn global_types_from_names(schema: &Schema, global_names: &HashSet<String>) -> Result<Vec<String>> {
    // We want to add names that are referenced by input objects, recursively.
    let mut name_to_type = HashMap::with_capacity(global_names.len());
    for name in global_names {
        let global_type = schema
            .get_type_for_name(name)
            .ok_or_else(|| Error::MissingType(name.to_string()))?;
        add_sub_input_objects(&mut name_to_type, schema, global_type)?;
        name_to_type.insert(name, global_type);
    }
    let mut types = Vec::new();
    let mut sorted_names = name_to_type.iter().collect::<Vec<_>>();
    sorted_names.sort_unstable_by_key(|value| value.0);
    for (name, global_type) in sorted_names {
        match &global_type.definition {
            TypeDefinition::Enum(enum_type) => {
                types.push(enum_def_from_type(
                    name,
                    &global_type.documentation,
                    enum_type,
                ));
            }
            TypeDefinition::InputObject(input_object_type) => {
                types.push(input_def_from_type(input_object_type)?);
            }
            _ => return Err(Error::NotGlobalType(name.to_string())),
        }
    }
    Ok(types)
}

pub fn compile_globals(
    schema: &Schema,
    global_names: &HashSet<String>,
) -> Result<GlobalTypesCompile> {
    let types = global_types_from_names(schema, global_names)?;
    let contents = format!("{}{}", HEADER, types.join("\n\n"));
    Ok(GlobalTypesCompile {
        filename: String::from("globalTypes.ts"),
        contents,
    })
}

pub struct CompileContext<'a> {
    schema: &'a Schema,
    global_types: HashSet<String>,
    imported_fragments: HashMap<String, query::FragmentDefinition>,
}

impl<'a> CompileContext<'a> {
    fn new(
        schema: &'a Schema,
        imported_fragments: HashMap<String, query::FragmentDefinition>,
    ) -> Self {
        CompileContext {
            schema,
            global_types: HashSet::new(),
            imported_fragments,
        }
    }

    fn compile_imports(&self) -> String {
        if self.global_types.is_empty() {
            return String::from("");
        }
        let mut names = self
            .global_types
            .iter()
            .map(|name| name.to_string())
            .collect::<Vec<String>>();
        names.sort();
        format!(
            "import {{ {} }} from \"__generated__/globalTypes\";\n\n",
            names.join(", ")
        )
    }

    fn add_type(&mut self, name: &str) {
        self.global_types.insert(name.to_string());
    }

    fn get_foreign_fragment(&self, name: &str) -> Option<&query::FragmentDefinition> {
        self.imported_fragments.get(name)
    }
}
