use super::graphql::ir;
use super::graphql::schema;
use super::graphql::schema::field as schema_field;
use super::graphql::variable;
use super::graphql::{BottomTypeConfig, CompileConfig};
use crate::cli::PrintableMessage;
use field::compile_scalar;
use std::collections::{HashMap, HashSet};

mod field;

const EMPTY: &str = "";
const HEADER: &str = "/* eslint-disable */
// This file was automatically generated and should not be edited.

";

#[derive(Debug)]
pub enum Error {
    MissingType(String),
    NotGlobalType(String),
    InvalidFieldDef {
        field_type_name: String,
        field_name: String,
    },
    ExpectedAtLeastOnePossibility,
}

impl From<Error> for PrintableMessage {
    fn from(error: Error) -> Self {
        match error {
            Error::MissingType(type_name) => PrintableMessage::new_simple_program_error(&format!(
                "failed lookup of type `{type_name}`",
            )),
            Error::NotGlobalType(type_name) => {
                PrintableMessage::new_simple_program_error(&format!(
                    "unexpected global type of `{type_name}`, which is not an enum nor input object",
                ))
            }
            Error::InvalidFieldDef {
                field_name,
                field_type_name,
            } => PrintableMessage::new_simple_program_error(&format!(
                "unexpected field `{field_name}` of type `{field_type_name}`: must be enum, another input object, or scalar",
            )),
            Error::ExpectedAtLeastOnePossibility => PrintableMessage::new_simple_program_error(
                "could not determine possiblities for complex type",
            ),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

type Typescript = String;

#[derive(Debug)]
pub struct GlobalTypesCompile {
    pub filename: String,
    pub contents: String,
}

#[derive(Debug)]
pub struct Compile {
    pub filename: String,
    pub contents: String,
    pub global_types_used: HashSet<String>,
}

fn from_input_def_field_def(
    config: &CompileConfig,
    field_name: &str,
    field: &schema_field::Field,
) -> Result<String> {
    let concrete = &field.type_description.reveal_concrete();
    let output = match &concrete.definition {
        schema_field::FieldTypeDefinition::Scalar(sc_type) => compile_scalar(config, sc_type),
        schema_field::FieldTypeDefinition::Enum => concrete.name.to_string(),
        schema_field::FieldTypeDefinition::InputObject => concrete.name.to_string(),
        _ => {
            return Err(Error::InvalidFieldDef {
                field_type_name: concrete.name.clone(),
                field_name: field_name.to_string(),
            })
        }
    };
    Ok(output)
}

fn input_def_from_type(
    config: &CompileConfig,
    input_type: &schema::InputObjectType,
) -> Result<String> {
    let mut fields = Vec::new();
    // For test and file signature stability
    let mut sorted = input_type.fields.iter().collect::<Vec<_>>();
    sorted.sort_unstable_by_key(|item| item.0);
    for (name, field) in sorted.into_iter() {
        let doc = compile_documentation(&field.documentation, field.deprecated, 2);
        let field_type = from_input_def_field_def(config, name, field)?;
        let (_, last_type_mod) = field.type_description.type_modifiers();
        let ts_field = match last_type_mod {
            schema_field::FieldTypeModifier::None => format!("  {doc}{name}: {field_type};"),
            schema_field::FieldTypeModifier::Nullable => {
                format!("  {doc}{name}?: {field_type} | null;")
            }
            schema_field::FieldTypeModifier::NullableList => {
                format!("  {doc}{name}?: {field_type}[] | null;")
            }
            schema_field::FieldTypeModifier::List => {
                format!("  {doc}{name}: {field_type}[];")
            }
            schema_field::FieldTypeModifier::ListOfNullable => {
                format!("  {doc}{name}: ({field_type} | null)[];")
            }
            schema_field::FieldTypeModifier::NullableListOfNullable => {
                format!("  {doc}{name}?: ({field_type} | null)[] | null;")
            }
        };
        fields.push(ts_field);
    }
    Ok(format!(
        "export type {} = {{\n{}\n}};",
        input_type.name,
        fields.join("\n")
    ))
}

fn enum_def_from_type(
    name: &str,
    documentation: &schema::Documentation,
    enum_type: &schema::EnumType,
) -> String {
    let doc_comment = compile_documentation(documentation, false, 0);
    let mut sorted_values = enum_type
        .possible_values
        .iter()
        .map(|value| format!("  {value} = \"{value}\","))
        .collect::<Vec<String>>();
    sorted_values.sort_unstable();
    let joined = sorted_values.join("\n");
    format!("{doc_comment}export enum {name} {{\n{joined}\n}}")
}

fn add_sub_input_objects<'a>(
    name_to_type: &mut HashMap<&'a str, &'a schema::Type>,
    schema: &'a schema::Schema,
    current_type: &'a schema::Type,
) -> Result<()> {
    if let schema::TypeDefinition::InputObject(input_object_type) = &current_type.definition {
        for field in input_object_type.fields.values() {
            add_sub_input_object_field(name_to_type, schema, &field.type_description, &field.name)?;
        }
    }
    Ok(())
}

fn add_sub_input_object_field<'a>(
    name_to_type: &mut HashMap<&'a str, &'a schema::Type>,
    schema: &'a schema::Schema,
    field_type: &'a schema_field::FieldType,
    field_name: &'a str,
) -> Result<()> {
    let concrete = field_type.reveal_concrete();
    let type_name = &concrete.name;
    match concrete.definition {
        schema_field::FieldTypeDefinition::InputObject => {
            let global_type = schema
                .get_type_for_name(type_name)
                .ok_or_else(|| Error::MissingType(type_name.to_string()))?;
            name_to_type.insert(type_name, global_type);
            add_sub_input_objects(name_to_type, schema, global_type)?;
        }
        schema_field::FieldTypeDefinition::Enum => {
            let global_type = schema
                .get_type_for_name(type_name)
                .ok_or_else(|| Error::MissingType(type_name.to_string()))?;
            name_to_type.insert(type_name, global_type);
        }
        schema_field::FieldTypeDefinition::Scalar(_) => {}
        _ => {
            return Err(Error::InvalidFieldDef {
                field_type_name: type_name.clone(),
                field_name: field_name.to_string(),
            })
        }
    }
    Ok(())
}

fn global_types_from_names(
    config: &CompileConfig,
    schema: &schema::Schema,
    global_names: &HashSet<String>,
) -> Result<Vec<String>> {
    // We want to add names that are referenced by input objects, recursively.
    let mut name_to_type = HashMap::with_capacity(global_names.len());
    for name in global_names {
        let global_type = schema
            .get_type_for_name(name)
            .ok_or_else(|| Error::MissingType(name.to_string()))?;
        add_sub_input_objects(&mut name_to_type, schema, global_type)?;
        name_to_type.insert(name, global_type);
    }
    // For test and file signature stability
    let mut sorted_names = name_to_type.iter().collect::<Vec<_>>();
    sorted_names.sort_unstable_by_key(|value| value.0);
    sorted_names
        .into_iter()
        .map(|(name, global_type)| {
            let def = match &global_type.definition {
                schema::TypeDefinition::Enum(enum_type) => {
                    enum_def_from_type(name, &global_type.documentation, enum_type)
                }
                schema::TypeDefinition::InputObject(input_object_type) => {
                    input_def_from_type(config, input_object_type)?
                }
                _ => return Err(Error::NotGlobalType((*name).to_string())),
            };
            Ok(def)
        })
        .collect()
}

pub fn compile_globals(
    config: &CompileConfig,
    schema: &schema::Schema,
    global_names: &HashSet<String>,
) -> Result<GlobalTypesCompile> {
    let type_definitions = global_types_from_names(config, schema, global_names)?;
    Ok(GlobalTypesCompile {
        filename: format!("{}.ts", config.global_types_module_name),
        contents: format!("{HEADER}{}", type_definitions.join("\n\n")),
    })
}

fn compile_documentation(
    documentation: &schema::Documentation,
    deprecated: bool,
    tab_width: usize,
) -> Typescript {
    let tab = " ".repeat(tab_width);

    let processed_documentation = documentation.as_deref().map(|docs| {
        docs.replace('\n', &format!("\n {tab}* "))
            .replace("/*", EMPTY)
            .replace("*/", EMPTY)
    });

    let wrap = |content: &str| format!("/**\n {tab}* {content}\n {tab}*/\n{tab}");

    match (processed_documentation.as_deref(), deprecated) {
        (Some(docs), true) => wrap(&format!("{docs}\n {tab}* @deprecated")),
        (Some(docs), false) => wrap(docs),
        (None, true) => wrap("@deprecated"),
        (None, false) => EMPTY.to_string(),
    }
}

fn compile_custom_scalar_name(
    config: &CompileConfig,
    string_thing: impl std::string::ToString,
) -> Typescript {
    match &config.bottom_type_config {
        BottomTypeConfig::DefaultBottomType => String::from("any"),
        BottomTypeConfig::RealName => string_thing.to_string(),
        BottomTypeConfig::RealNameWithPrefix(s) => format!("{s}{}", string_thing.to_string()),
    }
}

fn type_name_from_scalar(config: &CompileConfig, scalar: &schema_field::ScalarType) -> Typescript {
    match scalar {
        schema_field::ScalarType::Boolean => String::from("boolean"),
        schema_field::ScalarType::String | schema_field::ScalarType::Id => String::from("string"),
        schema_field::ScalarType::Float | schema_field::ScalarType::Int => String::from("number"),
        schema_field::ScalarType::Custom(name) => compile_custom_scalar_name(config, name),
    }
}

fn prop_type_def(
    type_modifier: &schema_field::FieldTypeModifier,
    flat_type_name: String,
) -> String {
    match type_modifier {
        schema_field::FieldTypeModifier::None => flat_type_name,
        schema_field::FieldTypeModifier::Nullable => format!("{flat_type_name} | null"),
        schema_field::FieldTypeModifier::List => format!("{flat_type_name}[]"),
        schema_field::FieldTypeModifier::NullableList => format!("{flat_type_name}[] | null"),
        schema_field::FieldTypeModifier::ListOfNullable => format!("({flat_type_name} | null)[]"),
        schema_field::FieldTypeModifier::NullableListOfNullable => {
            format!("({flat_type_name} | null)[] | null")
        }
    }
}

fn field_ids_for_complex_ir(complex: &ir::Complex) -> HashSet<&str> {
    complex
        .fields
        .iter()
        .map(|field_ir| field_ir.prop_name.as_ref())
        .collect()
}

fn type_definitions_from_smoosh_complex_ir<'a>(
    config: &CompileConfig,
    global_types: &mut HashSet<String>,
    complex: &ir::Complex,
    possibilities: impl Iterator<Item = &'a str>,
    smooth_type_name: &str,
) -> Result<Vec<Typescript>> {
    let mut modified_complex = complex.clone();
    // For test and file signature stability
    let mut sorted = possibilities.collect::<Vec<_>>();
    sorted.sort_unstable();
    modified_complex.name = sorted.join("\" | \"");
    type_definitions_from_complex_ir(config, global_types, &modified_complex, smooth_type_name)
}

fn type_definitions_from_complex_field_collection(
    config: &CompileConfig,
    global_types: &mut HashSet<String>,
    collection: &ir::ComplexCollection,
    main_type: &str,
) -> Result<Vec<Typescript>> {
    let num_possible_types = collection.possibilities.len();
    if num_possible_types == 0 {
        return Err(Error::ExpectedAtLeastOnePossibility);
    }

    let first_possiblity = &collection.possibilities[0];
    if num_possible_types == 1 {
        // Special case of single possibility, we don't need the discriminating type
        return type_definitions_from_complex_ir(config, global_types, first_possiblity, main_type);
    }

    // Let's try to reduce output and improve readability and convenience by outputing a "smooshed type"
    // of common/completely repeated properties, since this is so common for interface types.
    let mut common_props = field_ids_for_complex_ir(first_possiblity);
    let mut all_fields = Vec::with_capacity(collection.possibilities.len());
    all_fields.push((&first_possiblity.name[..], common_props.clone()));
    for possibility in &collection.possibilities[1..] {
        let pos_props = field_ids_for_complex_ir(possibility);
        common_props.retain(|prop_name| pos_props.contains(prop_name));
        all_fields.push((&possibility.name[..], pos_props));
    }

    // These will represent all the types can can be smooshed down
    let repeated_possiblities: HashSet<&str> = all_fields
        .into_iter()
        .filter(|(_, fields)| common_props == *fields)
        .map(|(name, _)| name)
        .collect();

    let num_repeats = repeated_possiblities.len();
    if num_possible_types == num_repeats {
        // If there is _only_ a smoosh type, just compile the smoosh type _as_ the main type
        return type_definitions_from_smoosh_complex_ir(
            config,
            global_types,
            first_possiblity,
            repeated_possiblities.into_iter(),
            main_type,
        );
    }

    let mut definitions = Vec::new();
    let mut names = Vec::new();
    let mut maybe_common_representative: Option<&ir::Complex> = None;
    for possibility in collection.possibilities.iter() {
        if repeated_possiblities.contains(&possibility.name[..]) {
            maybe_common_representative = Some(possibility);
            continue;
        }
        let possiblity_prop_path = format!("{main_type}_{}", possibility.name);
        definitions.append(&mut type_definitions_from_complex_ir(
            config,
            global_types,
            possibility,
            &possiblity_prop_path,
        )?);
        names.push(possiblity_prop_path);
    }

    if let Some(common_representative) = maybe_common_representative {
        let smoosh_type_name = format!("{main_type}_$$other");
        definitions.append(&mut type_definitions_from_smoosh_complex_ir(
            config,
            global_types,
            common_representative,
            repeated_possiblities.into_iter(),
            &smoosh_type_name,
        )?);
        names.push(smoosh_type_name);
    }
    definitions.push(format!("export type {main_type} = {};", names.join(" | ")));
    Ok(definitions)
}

fn type_definitions_from_complex_ir<'a>(
    config: &CompileConfig,
    global_types: &mut HashSet<String>,
    complex_ir: &'a ir::Complex,
    prop_path: &str,
) -> Result<Vec<Typescript>> {
    let mut definitions = Vec::new();
    let mut prop_defs = Vec::new();
    for field_ir in &complex_ir.fields {
        let flat_type_name = match &field_ir.type_ir {
            ir::FieldType::Complex(complex_collection) => {
                let sub_prop_path = format!("{prop_path}_{}", field_ir.prop_name);
                definitions.extend(type_definitions_from_complex_field_collection(
                    config,
                    global_types,
                    complex_collection,
                    &sub_prop_path,
                )?);
                sub_prop_path
            }
            ir::FieldType::Enum(name) => {
                global_types.insert(name.clone());
                name.clone()
            }
            ir::FieldType::Scalar(scalar_type) => type_name_from_scalar(config, scalar_type),
            ir::FieldType::TypeName => format!("\"{}\"", complex_ir.name),
        };
        let prop_def_type = prop_type_def(&field_ir.last_type_modifier, flat_type_name);
        let doc_comment = compile_documentation(&field_ir.documentation, field_ir.deprecated, 2);
        let readonly_modifier = if config.use_readonly_types {
            "readonly "
        } else {
            EMPTY
        };
        prop_defs.push(format!(
            "  {doc_comment}{readonly_modifier}{}: {prop_def_type};",
            field_ir.prop_name,
        ));
    }
    definitions.push(format!(
        "export type {prop_path} = {{\n{}\n}};",
        prop_defs.join("\n")
    ));
    Ok(definitions)
}

fn compile_variable_type_name<'a>(
    config: &CompileConfig,
    schema: &schema::Schema,
    global_types: &mut HashSet<String>,
    var_ir: &variable::Variable<'a>,
) -> Result<Typescript> {
    let type_name = match var_ir.type_ir {
        variable::VariableType::Id | variable::VariableType::String => "string".to_string(),
        variable::VariableType::Float | variable::VariableType::Int => "number".to_string(),
        variable::VariableType::Boolean => "boolean".to_string(),
        variable::VariableType::Custom(name) => {
            let global_type = schema
                .get_type_for_name(name)
                .ok_or_else(|| Error::MissingType(name.to_string()))?;
            match &global_type.definition {
                schema::TypeDefinition::Scalar(_) => compile_custom_scalar_name(config, name),

                _ => {
                    global_types.insert((*name).to_string());
                    name.to_string()
                }
            }
        }
    };
    Ok(type_name)
}

fn compile_variables_type_definition<'a>(
    config: &CompileConfig,
    schema: &schema::Schema,
    global_types: &mut HashSet<String>,
    op_ir: &ir::Operation<'a>,
) -> Result<Typescript> {
    op_ir
        .variables
        .as_ref()
        .map(|var_irs| {
            var_irs
                .iter()
                .map(|var_ir| {
                    let type_name =
                        compile_variable_type_name(config, schema, global_types, var_ir)?;
                    let type_def = prop_type_def(&var_ir.type_modifier, type_name);
                    Ok((var_ir, type_def))
                })
                .collect::<Result<Vec<(&variable::Variable<'a>, String)>>>()
                .map(|mut values| {
                    values.sort_unstable_by_key(|(var_ir, _type_def)| &var_ir.prop_name);
                    values
                        .into_iter()
                        .map(|(var_ir, type_def)| match &var_ir.type_modifier {
                            schema_field::FieldTypeModifier::Nullable
                            | schema_field::FieldTypeModifier::NullableList
                            | schema_field::FieldTypeModifier::NullableListOfNullable => {
                                format!("  {}?: {type_def};", var_ir.prop_name)
                            }
                            _ => format!("  {}: {type_def};", var_ir.prop_name),
                        })
                        .collect::<Vec<String>>()
                        .join("\n")
                })
        })
        .transpose()
        .map(|result| {
            result
                .map(|prop_defs| {
                    format!(
                        "\n\nexport type {}Variables = {{\n{}\n}};",
                        op_ir.name, prop_defs
                    )
                })
                .unwrap_or_else(|| EMPTY.to_string())
        })
}

fn compile_imports(config: &CompileConfig, used_globals: &HashSet<String>) -> Typescript {
    if used_globals.is_empty() {
        return String::from(EMPTY);
    }
    // For test and file signature stability
    let mut sorted_names: Vec<&str> = used_globals.iter().map(|g| g.as_ref()).collect();
    sorted_names.sort_unstable();
    format!(
        "import type {{ {} }} from \"{}{}/{}\";\n\n",
        sorted_names.join(", "),
        config.root_dir_import_prefix.as_deref().unwrap_or(EMPTY),
        config.generated_module_name,
        config.global_types_module_name,
    )
}

pub fn compile_ir(
    op_ir: &ir::Operation<'_>,
    config: &CompileConfig,
    schema: &schema::Schema,
) -> Result<Compile> {
    let mut global_types_used = HashSet::new();
    let type_definitions = type_definitions_from_complex_field_collection(
        config,
        &mut global_types_used,
        &op_ir.collection,
        &op_ir.name,
    )?;
    let variable_type_def =
        compile_variables_type_definition(config, schema, &mut global_types_used, op_ir)?;
    let imports = compile_imports(config, &global_types_used);
    Ok(Compile {
        filename: format!("{}.ts", op_ir.name),
        contents: format!(
            "{HEADER}{imports}{}{variable_type_def}",
            type_definitions.join("\n\n"),
        ),
        global_types_used,
    })
}
