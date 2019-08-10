use super::field::{compile_ts_fields, TypeNameDescription};
use super::{CompileContext, Error, Parent, Result, Typescript};
use crate::graphql::schema::{Field, FieldsLookup, TypeDefinition};
use graphql_parser::query::{Selection, SelectionSet, TypeCondition};
use std::collections::{HashMap, HashSet};

type TSInterfaces = Vec<Typescript>;

#[derive(Debug)]
pub struct FieldIR {
    pub field: Field,
    pub user_specified_name: String,
    pub selections: FieldsIR,
}

impl FieldIR {
    fn has_selections(&self) -> bool {
        self.selections.has_selections()
    }
}

pub type FieldsKey = (String, String);

#[derive(Debug)]
pub struct FieldsIR {
    pub collection: HashMap<FieldsKey, FieldIR>,
}

impl FieldsIR {
    fn with_capacity(size: usize) -> Self {
        FieldsIR {
            collection: HashMap::with_capacity(size),
        }
    }

    fn has_selections(&self) -> bool {
        !self.collection.is_empty()
    }

    fn insert(
        &mut self,
        parent_type_name: &str,
        inner_fields_ir: FieldsIR,
        user_specified_name: String,
        field: &Field,
    ) {
        let key = (parent_type_name.to_string(), user_specified_name.clone());
        match self.collection.get_mut(&key) {
            Some(f_ir) => {
                f_ir.selections.merge(inner_fields_ir);
            }
            None => {
                let new_field_ir = FieldIR {
                    field: field.clone(),
                    user_specified_name,
                    selections: inner_fields_ir,
                };
                self.collection.insert(key, new_field_ir);
            }
        }
    }

    fn merge(&mut self, mut other: FieldsIR) {
        for (key, field_ir) in other.collection.drain() {
            match self.collection.get_mut(&key) {
                Some(inner) => {
                    inner.selections.merge(field_ir.selections);
                }
                None => {
                    self.collection.insert(key, field_ir);
                }
            }
        }
    }
}
fn compile_sums_and_products(
    sum_types: Vec<String>,
    product_types: Vec<String>,
    parent: &Parent,
) -> Result<Typescript> {
    let sums = sum_types.join(" | ");
    let products = product_types.join(" & ");
    let rh_def = match (sums.is_empty(), products.is_empty()) {
        (true, true) => return Err(Error::UnknownError),
        (true, false) => products,
        (false, true) => sums,
        (false, false) => format!("({}) & {}", sums, products),
    };
    Ok(format!(
        "export type {} = {};",
        parent.compiled_name, rh_def
    ))
}

fn compile_implementors(
    ctx: &mut CompileContext,
    fields: &[&FieldIR],
    parent: &Parent,
    typename_description: Option<&TypeNameDescription>,
) -> Result<TSInterfaces> {
    let compiled_fields_info = compile_ts_fields(ctx, parent, fields, typename_description)?;
    let mut ts_interfaces = Vec::new();
    let mut compiled_fields_ts = String::new();
    for (compiled_name, opt_sub_field, compiled_field) in &compiled_fields_info {
        compiled_fields_ts.push_str("\n");
        compiled_fields_ts.push_str(&compiled_field);
        let sub_field = match opt_sub_field {
            Some(s) => s,
            _ => continue,
        };
        if !sub_field.has_selections() {
            continue;
        }
        let new_parent = Parent {
            compiled_name: compiled_name.clone(),
            type_name: sub_field.field.type_description.name.clone(),
        };
        ts_interfaces.append(&mut compile_fields_ir(
            ctx,
            &sub_field.selections,
            &new_parent,
        )?);
    }
    ts_interfaces.push(format!(
        "export interface {} {{{}\n}}",
        parent.compiled_name, compiled_fields_ts,
    ));
    Ok(ts_interfaces)
}

fn get_type_name_descripton_for_interface<'a>(
    interface_name: &str,
    implementing_types: &HashMap<&String, (&TypeDefinition, Vec<&'a FieldIR>)>,
) -> TypeNameDescription {
    let union_implmentor = implementing_types
        .values()
        .find(|(type_def, _)| match type_def {
            TypeDefinition::Interface(interface_type) => interface_type.name == interface_name,
            _ => false,
        });
    union_implmentor
        .map(|(_, sub_fields)| {
            let mut names = HashSet::new();
            for sub_field in sub_fields {
                let name = sub_field.user_specified_name.clone();
                if sub_field.field.name != "__typename" {
                    continue;
                }
                names.insert(name);
            }
            TypeNameDescription {
                aliases: names,
                opt_possible_types: None,
            }
        })
        .unwrap_or_else(|| TypeNameDescription {
            aliases: HashSet::new(),
            opt_possible_types: None,
        })
}

fn get_type_name_descripton_for_union<'a>(
    union_name: &str,
    implementing_types: &HashMap<&String, (&TypeDefinition, Vec<&'a FieldIR>)>,
) -> Result<TypeNameDescription> {
    let union_implmentor = implementing_types
        .values()
        .find(|(type_def, _)| match type_def {
            TypeDefinition::Union(union_type) => union_type.name == union_name,
            _ => false,
        });
    let aliases = union_implmentor
        .map(|(_, sub_fields)| {
            let mut names = HashSet::new();
            for sub_field in sub_fields {
                let name = sub_field.user_specified_name.clone();
                if sub_field.field.name != "__typename" {
                    return Err(Error::UnionOnlyAllowedTypename(name));
                }
                names.insert(name);
            }
            Ok(names)
        })
        .unwrap_or_else(|| {
            let mut names = HashSet::new();
            names.insert("__typename".to_string());
            Ok(names)
        })?;
    Ok(TypeNameDescription {
        aliases,
        opt_possible_types: None,
    })
}

fn combine_typename_descriptions(
    base_desc: &TypeNameDescription,
    fields: &[&FieldIR],
) -> TypeNameDescription {
    let mut from_fields = fields
        .iter()
        .filter(|field_ir| field_ir.field.name == "__typename")
        .map(|field_ir| field_ir.user_specified_name.clone())
        .collect::<HashSet<_>>();
    for alias in base_desc.aliases.iter() {
        from_fields.insert(alias.clone());
    }
    TypeNameDescription {
        aliases: from_fields,
        opt_possible_types: None,
    }
}

fn compile_fields_ir(
    ctx: &mut CompileContext,
    fields_ir: &FieldsIR,
    parent: &Parent,
) -> Result<TSInterfaces> {
    let mut implementing_types = HashMap::new();
    for (key, value) in &fields_ir.collection {
        let type_name = &key.0;
        let parent_type = ctx
            .schema
            .get_type_for_name(type_name)
            .ok_or_else(|| Error::MissingType(type_name.to_string()))?;
        let (_, sub_fields) = implementing_types
            .entry(type_name)
            .or_insert_with(|| (&parent_type.definition, vec![]));
        sub_fields.push(value);
    }
    let mut ts_interfaces = Vec::new();
    let parent_type = ctx
        .schema
        .get_type_for_name(&parent.type_name)
        .ok_or_else(|| Error::MissingType(parent.type_name.to_string()))?;
    match &parent_type.definition {
        TypeDefinition::Object(_) => match implementing_types.get(&parent.type_name) {
            None => {}
            Some((_, sub_fields)) => {
                ts_interfaces.append(&mut compile_implementors(ctx, sub_fields, &parent, None)?);
            }
        },
        TypeDefinition::Union(union_type) => {
            let mut sum_types = Vec::new();
            let mut product_types = Vec::new();
            let type_name_description =
                get_type_name_descripton_for_union(&union_type.name, &implementing_types)?;
            for type_name in &union_type.possible_types {
                let union_implementor_type = ctx
                    .schema
                    .get_type_for_name(type_name)
                    .ok_or_else(|| Error::MissingType(type_name.to_string()))?;
                let compiled_name = format!("{}_{}", parent.compiled_name, type_name);
                let new_parent = Parent {
                    compiled_name: compiled_name.clone(),
                    type_name: type_name.to_string(),
                };
                sum_types.push(compiled_name);
                match (
                    &union_implementor_type.definition,
                    implementing_types.get(type_name).map(|x| &x.1),
                ) {
                    (TypeDefinition::Object(_), Some(sub_fields)) => {
                        let combined =
                            combine_typename_descriptions(&type_name_description, &sub_fields);
                        ts_interfaces.append(&mut compile_implementors(
                            ctx,
                            sub_fields,
                            &new_parent,
                            Some(&combined),
                        )?);
                    }
                    (TypeDefinition::Object(_), None) => {
                        ts_interfaces.append(&mut compile_implementors(
                            ctx,
                            &[],
                            &new_parent,
                            Some(&type_name_description),
                        )?);
                    }
                    _ => return Err(Error::UnionCanOnlyHaveObjectImplementors),
                }
            }

            let mut sorted_implementing_type_names = implementing_types
                .keys()
                .filter(|name| !union_type.possible_types.contains(name))
                .collect::<Vec<_>>();
            sorted_implementing_type_names.sort_unstable();
            for type_name in sorted_implementing_type_names {
                let (type_def, sub_fields) = implementing_types.get(type_name).unwrap();
                let compiled_name = format!("{}_{}", parent.compiled_name, type_name);
                let new_parent = Parent {
                    compiled_name: compiled_name.clone(),
                    type_name: type_name.to_string(),
                };
                match &type_def {
                    TypeDefinition::Union(_) => {}
                    TypeDefinition::Interface(_) => {
                        product_types.push(compiled_name);
                        ts_interfaces.append(&mut compile_implementors(
                            ctx,
                            sub_fields,
                            &new_parent,
                            None,
                        )?);
                    }
                    _ => return Err(Error::UnknownError),
                }
            }
            ts_interfaces.push(compile_sums_and_products(sum_types, product_types, parent)?);
        }
        TypeDefinition::Interface(interface_type) => {
            let mut sorted_implementing_type_names = implementing_types.keys().collect::<Vec<_>>();
            sorted_implementing_type_names.sort_unstable();
            let type_name_description =
                get_type_name_descripton_for_interface(&interface_type.name, &implementing_types);
            let mut sum_types = Vec::new();
            let mut product_types = Vec::new();
            for name in sorted_implementing_type_names {
                let (type_def, sub_fields) = implementing_types.get(name).unwrap();
                let compiled_name = format!("{}_{}", parent.compiled_name, name);
                let new_parent = Parent {
                    compiled_name: compiled_name.clone(),
                    type_name: name.to_string(),
                };
                match &type_def {
                    TypeDefinition::Object(_) => {
                        sum_types.push(compiled_name);
                        let combined =
                            combine_typename_descriptions(&type_name_description, &sub_fields);
                        ts_interfaces.append(&mut compile_implementors(
                            ctx,
                            sub_fields,
                            &new_parent,
                            Some(&combined),
                        )?);
                    }
                    TypeDefinition::Interface(interface_type) => {
                        if sub_fields.iter().all(|x| x.field.name == "__typename") {
                            continue;
                        }
                        let typename_desc = TypeNameDescription {
                            aliases: sub_fields
                                .iter()
                                .filter(|x| x.field.name == "__typename")
                                .map(|x| x.user_specified_name.clone())
                                .collect(),
                            opt_possible_types: Some(interface_type.possible_types.clone()),
                        };
                        product_types.push(compiled_name);
                        ts_interfaces.append(&mut compile_implementors(
                            ctx,
                            sub_fields,
                            &new_parent,
                            Some(&typename_desc),
                        )?);
                    }
                    _ => return Err(Error::UnknownError),
                }
            }
            ts_interfaces.push(compile_sums_and_products(sum_types, product_types, parent)?);
        }
        _ => return Err(Error::SelectionSetOnWrongType(parent.type_name.clone())),
    }
    Ok(ts_interfaces)
}

fn inner_fields_ir_from_fragment(
    ctx: &mut CompileContext,
    fragment_type_name: &str,
    fragment_selections: &[Selection],
) -> Result<FieldsIR> {
    let spread_type = ctx
        .schema
        .get_type_for_name(fragment_type_name)
        .ok_or_else(|| Error::MissingType(fragment_type_name.to_string()))?;
    let type_fields = spread_type.definition.get_fields_lookup();
    traverse_selection_set_items(ctx, &fragment_selections, type_fields, fragment_type_name)
}

fn traverse_selection_set_items(
    ctx: &mut CompileContext,
    selections: &[Selection],
    fields: Option<&FieldsLookup>,
    parent_type_name: &str,
) -> Result<FieldsIR> {
    let mut fields_ir = FieldsIR::with_capacity(selections.len());
    for selection in selections {
        match selection {
            Selection::Field(selection_field) => {
                let field = fields
                    .ok_or_else(|| Error::SelectionSetOnWrongType(parent_type_name.to_string()))?
                    .get(&selection_field.name)
                    .ok_or_else(|| {
                        Error::UnknownField(
                            parent_type_name.to_string(),
                            selection_field.name.to_string(),
                        )
                    })?;
                let field_type_name = &field.type_description.name;
                let field_type = ctx
                    .schema
                    .get_type_for_name(field_type_name)
                    .ok_or_else(|| Error::MissingType(field.type_description.name.clone()))?;
                let field_fields = field_type.definition.get_fields_lookup();
                let inner_fields_ir = traverse_selection_set_items(
                    ctx,
                    &selection_field.selection_set.items,
                    field_fields,
                    field_type_name,
                )?;
                let user_specified_name = selection_field
                    .alias
                    .clone()
                    .unwrap_or_else(|| selection_field.name.to_string());
                fields_ir.insert(
                    parent_type_name,
                    inner_fields_ir,
                    user_specified_name,
                    field,
                );
            }
            Selection::InlineFragment(fragment_def) => {
                let type_name = match &fragment_def.type_condition {
                    Some(TypeCondition::On(name)) => name,
                    _ => return Err(Error::MissingTypeCondition),
                };
                fields_ir.merge(inner_fields_ir_from_fragment(
                    ctx,
                    type_name,
                    &fragment_def.selection_set.items,
                )?);
            }
            Selection::FragmentSpread(spread) => {
                let fragment_def = ctx
                    .get_foreign_fragment(&spread.fragment_name)
                    .ok_or_else(|| Error::UnknownFragment(spread.fragment_name.clone()))?
                    .clone();
                let TypeCondition::On(type_name) = &fragment_def.type_condition;
                fields_ir.merge(inner_fields_ir_from_fragment(
                    ctx,
                    type_name,
                    &fragment_def.selection_set.items,
                )?);
            }
        }
    }
    Ok(fields_ir)
}

pub fn from_selection_set(
    ctx: &mut CompileContext,
    selection_set: &SelectionSet,
    parent: &Parent,
) -> Result<TSInterfaces> {
    let parent_type = ctx
        .schema
        .get_type_for_name(&parent.type_name)
        .ok_or_else(|| Error::MissingType(parent.type_name.to_string()))?;
    let fields = parent_type
        .definition
        .get_fields_lookup()
        .ok_or_else(|| Error::SelectionSetOnWrongType(parent.type_name.clone()))?;
    let fields_ir =
        traverse_selection_set_items(ctx, &selection_set.items, Some(fields), &parent.type_name)?;
    let ts_interfaces = compile_fields_ir(ctx, &fields_ir, parent)?;
    Ok(ts_interfaces)
}
