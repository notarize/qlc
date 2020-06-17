use super::graphql::ir;
use super::graphql::schema;
use super::graphql::schema::field as schema_field;
use super::graphql::variable;
use super::graphql::{BottomTypeConfig, CompileConfig};
use field::compile_scalar;
use std::collections::{HashMap, HashSet};

mod field;

const HEADER: &str = "/* tslint:disable */
/* eslint-disable */
// This file was automatically generated and should not be edited.

";

#[derive(Debug)]
pub enum Error {
    MissingType(String),
    NotGlobalType(String),
    InvalidFieldDef(String),
    OutputInInput(String),
    ExpectedAtLeastOnePossibility,
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
    pub used_global_types: HashSet<String>,
}

fn from_input_def_field_def(
    config: &CompileConfig,
    field_name: &str,
    field: &schema_field::Field,
) -> Result<String> {
    let concrete = &field.type_description.reveal_concrete();
    let output = match &concrete.definition {
        schema_field::FieldTypeDefinition::Scalar(sc_type) => compile_scalar(config, &sc_type),
        schema_field::FieldTypeDefinition::Enum => concrete.name.to_string(),
        schema_field::FieldTypeDefinition::InputObject => concrete.name.to_string(),
        _ => return Err(Error::InvalidFieldDef(field_name.to_string())),
    };
    Ok(output)
}

fn input_def_from_type(
    config: &CompileConfig,
    input_type: &schema::InputObjectType,
) -> Result<String> {
    let mut fields = Vec::new();
    #[cfg(debug_assertions)] // for test stability, we sort here
    let fields_iter = {
        let mut sorted = input_type.fields.iter().collect::<Vec<_>>();
        sorted.sort_unstable_by_key(|item| item.0);
        sorted.into_iter()
    };
    #[cfg(not(debug_assertions))]
    let fields_iter = input_type.fields.iter();
    for (name, field) in fields_iter {
        let doc = compile_documentation(&field.documentation, 2);
        let field_type = from_input_def_field_def(config, name, &field)?;
        // TODO will this every be more than one item?
        let ts_field = match field.type_description.type_modifier_iter().last().unwrap() {
            schema_field::FieldTypeModifier::None => format!("  {}{}: {};", doc, name, field_type),
            schema_field::FieldTypeModifier::Nullable => {
                format!("  {}{}?: {} | null;", doc, name, field_type)
            }
            schema_field::FieldTypeModifier::NullableList => {
                format!("  {}{}?: {}[] | null;", doc, name, field_type)
            }
            schema_field::FieldTypeModifier::List => {
                format!("  {}{}: {}[];", doc, name, field_type)
            }
            schema_field::FieldTypeModifier::ListOfNullable => {
                format!("  {}{}: ({} | null)[];", doc, name, field_type)
            }
            schema_field::FieldTypeModifier::NullableListOfNullable => {
                format!("  {}{}?: ({} | null)[] | null;", doc, name, field_type)
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
        _ => return Err(Error::OutputInInput(field_name.to_string())),
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
    #[cfg(debug_assertions)] // for test stability
    let names = {
        let mut sorted_names = name_to_type.iter().collect::<Vec<_>>();
        sorted_names.sort_unstable_by_key(|value| value.0);
        sorted_names.into_iter()
    };
    #[cfg(not(debug_assertions))]
    let names = name_to_type.iter();
    names
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
        filename: String::from("globalTypes.ts"),
        contents: format!("{}{}", HEADER, type_definitions.join("\n\n")),
    })
}

fn compile_documentation(documentation: &schema::Documentation, tab_width: usize) -> Typescript {
    documentation
        .as_ref()
        .map(|docs| {
            let tab = " ".repeat(tab_width);
            let processed_desc = docs
                .replace("\n", &format!("\n {}* ", tab))
                .replace("*/", "");
            format!("/**\n {}* {}\n {}*/\n{}", tab, processed_desc, tab, tab,)
        })
        .unwrap_or_else(|| String::from(""))
}

fn type_name_from_scalar(config: &CompileConfig, scalar: &schema_field::ScalarType) -> Typescript {
    match scalar {
        schema_field::ScalarType::Boolean => String::from("boolean"),
        schema_field::ScalarType::String | schema_field::ScalarType::ID => String::from("string"),
        schema_field::ScalarType::Float | schema_field::ScalarType::Int => String::from("number"),
        schema_field::ScalarType::Custom(name) => match &config.bottom_type_config {
            BottomTypeConfig::UseBottomType => String::from("any"),
            BottomTypeConfig::UseRealName => name.clone(),
            BottomTypeConfig::UseRealNameWithPrefix(s) => format!("{}{}", s, name),
        },
    }
}

fn prop_type_def(
    type_modifier: &schema_field::FieldTypeModifier,
    flat_type_name: String,
) -> String {
    match type_modifier {
        schema_field::FieldTypeModifier::None => flat_type_name,
        schema_field::FieldTypeModifier::Nullable => format!("{} | null", flat_type_name),
        schema_field::FieldTypeModifier::List => format!("{}[]", flat_type_name),
        schema_field::FieldTypeModifier::NullableList => format!("{}[] | null", flat_type_name),
        schema_field::FieldTypeModifier::ListOfNullable => format!("({} | null)[]", flat_type_name),
        schema_field::FieldTypeModifier::NullableListOfNullable => {
            format!("({} | null)[] | null", flat_type_name)
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
    #[cfg(debug_assertions)] // for test stability
    let all_types = {
        let mut sorted = possibilities.collect::<Vec<_>>();
        sorted.sort_unstable();
        sorted
    };
    #[cfg(not(debug_assertions))]
    let all_types = possibilities.collect::<Vec<_>>();
    modified_complex.name = all_types.join("\" | \"");
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
            &first_possiblity,
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
        let possiblity_prop_path = format!("{}_{}", main_type, possibility.name);
        definitions.append(&mut type_definitions_from_complex_ir(
            config,
            global_types,
            possibility,
            &possiblity_prop_path,
        )?);
        names.push(possiblity_prop_path);
    }

    if let Some(common_representative) = maybe_common_representative {
        let smoosh_type_name = format!("{}_$$other", main_type);
        definitions.append(&mut type_definitions_from_smoosh_complex_ir(
            config,
            global_types,
            &common_representative,
            repeated_possiblities.into_iter(),
            &smoosh_type_name,
        )?);
        names.push(smoosh_type_name);
    }
    definitions.push(format!(
        "export type {} = {};",
        main_type,
        names.join(" | ")
    ));
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
                let sub_prop_path = format!("{}_{}", prop_path, field_ir.prop_name);
                definitions.extend(type_definitions_from_complex_field_collection(
                    config,
                    global_types,
                    &complex_collection,
                    &sub_prop_path,
                )?);
                sub_prop_path
            }
            ir::FieldType::Enum(name) => {
                global_types.insert(name.clone());
                name.clone()
            }
            ir::FieldType::Scalar(scalar_type) => type_name_from_scalar(config, &scalar_type),
            ir::FieldType::TypeName => format!("\"{}\"", complex_ir.name),
        };
        let prop_def_type = prop_type_def(&field_ir.type_modifiers.last().unwrap(), flat_type_name);
        let doc_comment = compile_documentation(&field_ir.documentation, 2);
        prop_defs.push(format!(
            "  {}{}: {};",
            doc_comment, field_ir.prop_name, prop_def_type
        ));
    }
    definitions.push(format!(
        "export type {} = {{\n{}\n}};",
        prop_path,
        prop_defs.join("\n")
    ));
    Ok(definitions)
}

fn compile_variables_type_definition(
    global_types: &mut HashSet<String>,
    op_ir: &ir::Operation<'_>,
) -> Result<Typescript> {
    let def = match &op_ir.variables {
        Some(var_irs) => {
            let prop_defs: Vec<_> = var_irs
                .iter()
                .map(|var_ir| {
                    let type_name = match &var_ir.type_ir {
                        variable::VariableType::ID | variable::VariableType::String => "string",
                        variable::VariableType::Float | variable::VariableType::Int => "number",
                        variable::VariableType::Boolean => "boolean",
                        variable::VariableType::Custom(name) => {
                            global_types.insert((*name).to_string());
                            name
                        }
                    };
                    let type_def = prop_type_def(&var_ir.type_modifier, type_name.to_string());
                    match &var_ir.type_modifier {
                        schema_field::FieldTypeModifier::Nullable
                        | schema_field::FieldTypeModifier::NullableList
                        | schema_field::FieldTypeModifier::NullableListOfNullable => {
                            format!("  {}?: {};", var_ir.prop_name, type_def)
                        }
                        _ => format!("  {}: {};", var_ir.prop_name, type_def),
                    }
                })
                .collect();
            format!(
                "\n\nexport type {}Variables = {{\n{}\n}};",
                op_ir.name,
                prop_defs.join("\n")
            )
        }
        None => "".into(),
    };
    Ok(def)
}

fn compile_imports(used_globals: &HashSet<String>) -> Typescript {
    if used_globals.is_empty() {
        return String::from("");
    }
    #[cfg(debug_assertions)] // for test stability
    let names = {
        let mut sorted: Vec<&str> = used_globals.iter().map(|g| g.as_ref()).collect();
        sorted.sort_unstable();
        sorted
    };
    #[cfg(not(debug_assertions))]
    let names: Vec<&str> = used_globals.iter().map(|g| g.as_ref()).collect();
    format!(
        "import {{ {} }} from \"__generated__/globalTypes\";\n\n",
        names.join(", ")
    )
}

pub fn compile_ir(op_ir: &ir::Operation<'_>, config: &CompileConfig) -> Result<Compile> {
    let mut used_global_types = HashSet::new();
    let type_definitions = type_definitions_from_complex_field_collection(
        config,
        &mut used_global_types,
        &op_ir.collection,
        &op_ir.name,
    )?;
    let variable_type_def = compile_variables_type_definition(&mut used_global_types, op_ir)?;
    let imports = compile_imports(&used_global_types);
    Ok(Compile {
        filename: format!("{}.ts", op_ir.name),
        contents: format!(
            "{}{}{}{}",
            HEADER,
            imports,
            type_definitions.join("\n\n"),
            variable_type_def
        ),
        used_global_types,
    })
}
