use crate::graphql::schema;
use crate::graphql::schema::field as schema_field;
use crate::graphql::variable;
use graphql_parser::query as parsed_query;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub enum Error {
    OperationUnsupported,
    UnparseableInputType,
    UnknownFragment(String),
    InputObjectOnSelection,
    MissingTypeCondition,
    MissingType(String),
    SelectionSetOnWrongType(String),
    MissingSelectionSetOnType(String),
    UnknownField(String, String),
    UnexpectedComplexTravseral(String),
    VariableError(variable::Error),
}

type Result<T> = std::result::Result<T, Error>;
type ImportedFragments = HashMap<String, parsed_query::FragmentDefinition>;

struct CompileContext<'a> {
    schema: &'a schema::Schema,
    imported_fragments: ImportedFragments,
}

/// Alias and field name
type FieldID<'a> = (&'a str, &'a str);

#[derive(Debug, Clone)]
struct UniqueFields<'a> {
    collection: HashMap<FieldID<'a>, (&'a schema_field::Field, FieldTraversal<'a>)>,
}

impl<'a> UniqueFields<'a> {
    fn new() -> Self {
        UniqueFields {
            collection: HashMap::new(),
        }
    }

    fn insert(
        &mut self,
        alias: &'a str,
        name: &'a str,
        field: &'a schema_field::Field,
        traversal: FieldTraversal<'a>,
    ) {
        match self.collection.entry((alias, name)) {
            Entry::Occupied(occupant) => {
                occupant.into_mut().1.extend_from(traversal);
            }
            Entry::Vacant(vacany) => {
                vacany.insert((field, traversal));
            }
        }
    }

    fn extend_from(&mut self, other: UniqueFields<'a>) {
        for ((alias, name), (field, traversal)) in other.collection.into_iter() {
            self.insert(alias, name, field, traversal);
        }
    }
}

impl<'a> TryFrom<UniqueFields<'a>> for Vec<Field> {
    type Error = Error;
    fn try_from(from: UniqueFields<'a>) -> Result<Self> {
        from.collection
            .into_iter()
            .map(|((alias, _name), (field, sub_traversal))| {
                let concrete = field.type_description.reveal_concrete();
                Ok(Field {
                    prop_name: alias.to_string(),
                    documentation: field.documentation.clone(),
                    type_modifiers: field
                        .type_description
                        .type_modifier_iter()
                        .cloned()
                        .collect(),
                    type_ir: get_type_ir_for_field(concrete, sub_traversal)?,
                })
            })
            .collect::<Result<Self>>()
            .map(|mut field_irs| {
                #[cfg(debug_assertions)] // for test stability
                field_irs.sort_unstable_by(|a, b| a.prop_name.cmp(&b.prop_name));
                field_irs
            })
    }
}

#[derive(Debug, Clone)]
struct TerminalTraversal<'a> {
    type_name: &'a str,
}

impl<'a> From<&'a str> for TerminalTraversal<'a> {
    fn from(type_name: &'a str) -> Self {
        TerminalTraversal { type_name }
    }
}

#[derive(Debug, Clone)]
struct ComplexTraversal<'a> {
    type_name: &'a str,
    fields_lookup: &'a schema::FieldsLookup,
    /// Concrete object type names mapped to its fields
    concrete_objects: HashMap<&'a str, UniqueFields<'a>>,
}

impl<'a> ComplexTraversal<'a> {
    fn clone_for_type_spread(
        &self,
        context: &'a CompileContext,
        spread_type_name: &'a str,
    ) -> Result<Self> {
        Self::try_from((context, spread_type_name)).map(|mut base| {
            // TODO what happens when we have zero? this is a good instance of warning
            base.concrete_objects
                .retain(|type_name, _| self.concrete_objects.contains_key(type_name));
            base
        })
    }

    fn insert_terminal(
        &mut self,
        alias: &'a str,
        name: &'a str,
        field: &'a schema_field::Field,
        terminal: TerminalTraversal<'a>,
    ) {
        for uniques in self.concrete_objects.values_mut() {
            uniques.insert(
                alias,
                name,
                field,
                FieldTraversal::Terminal(terminal.clone()),
            );
        }
    }

    fn insert_complex(
        &mut self,
        alias: &'a str,
        name: &'a str,
        field: &'a schema_field::Field,
        complex: ComplexTraversal<'a>,
    ) {
        for uniques in self.concrete_objects.values_mut() {
            uniques.insert(alias, name, field, FieldTraversal::Complex(complex.clone()));
        }
    }

    fn extend_from(&mut self, other: Self) {
        for (type_name, other_uniques) in other.concrete_objects.into_iter() {
            self.concrete_objects.get_mut(type_name).map(|uniques| {
                uniques.extend_from(other_uniques.clone());
                uniques
            });
        }
    }
}

impl<'a> TryFrom<ComplexTraversal<'a>> for ComplexCollection {
    type Error = Error;
    fn try_from(from: ComplexTraversal<'a>) -> Result<Self> {
        from.concrete_objects
            .into_iter()
            .map(|(name, uniques)| {
                Ok(Complex {
                    name: name.to_string(),
                    fields: uniques.try_into()?,
                })
            })
            .collect::<Result<Vec<_>>>()
            .map(|mut possibilities| {
                #[cfg(debug_assertions)] // for test stability
                possibilities.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                ComplexCollection { possibilities }
            })
    }
}

impl<'a> TryFrom<(&'a CompileContext<'a>, &'a str)> for ComplexTraversal<'a> {
    type Error = Error;
    fn try_from((context, type_name): (&'a CompileContext, &'a str)) -> Result<Self> {
        let schema_type = context
            .schema
            .get_type_for_name(type_name)
            .ok_or_else(|| Error::MissingType(type_name.to_string()))?;
        let fields_lookup = schema_type
            .definition
            .get_fields_lookup()
            .ok_or_else(|| Error::SelectionSetOnWrongType(type_name.to_string()))?;
        let concrete_objects = match &schema_type.definition {
            schema::TypeDefinition::Object(_) => {
                let mut concrete_objects = HashMap::with_capacity(1);
                concrete_objects.insert(type_name, UniqueFields::new());
                concrete_objects
            }
            schema::TypeDefinition::Union(union) => {
                let mut concrete_objects: HashMap<&str, _> =
                    HashMap::with_capacity(union.possible_types.len());
                for possible_type in union.possible_types.iter() {
                    concrete_objects.insert(possible_type, UniqueFields::new());
                }
                concrete_objects
            }
            schema::TypeDefinition::Interface(interface) => {
                let mut concrete_objects: HashMap<&str, _> =
                    HashMap::with_capacity(interface.possible_types.len());
                for possible_type in interface.possible_types.iter() {
                    concrete_objects.insert(possible_type, UniqueFields::new());
                }
                concrete_objects
            }
            _ => return Err(Error::UnexpectedComplexTravseral(type_name.to_string())),
        };
        Ok(ComplexTraversal {
            type_name,
            fields_lookup,
            concrete_objects,
        })
    }
}

#[derive(Debug, Clone)]
enum FieldTraversal<'a> {
    Terminal(TerminalTraversal<'a>),
    Complex(ComplexTraversal<'a>),
}

impl<'a> FieldTraversal<'a> {
    fn extend_from(&mut self, other: Self) {
        match self {
            Self::Complex(self_complex) => {
                if let Self::Complex(other_complex) = other {
                    self_complex.extend_from(other_complex);
                } else {
                    eprintln!("This is a bug in QLC (most likely):");
                    panic!(
                        "Cannot combine complex and terminal fields {}.",
                        self_complex.type_name
                    );
                }
            }
            Self::Terminal(self_terminal) => {
                if let Self::Complex(other_complex) = other {
                    eprintln!("This is a bug in QLC (most likely):");
                    panic!(
                        "Cannot combine terminal and complex fields {} and {}.",
                        self_terminal.type_name, other_complex.type_name
                    );
                }
                // Nothing to do if both terminal
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub prop_name: String,
    pub documentation: schema::Documentation,
    pub type_modifiers: Vec<schema_field::FieldTypeModifier>,
    pub type_ir: FieldType,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    TypeName,
    Complex(ComplexCollection),
    Enum(String),
    Scalar(schema_field::ScalarType),
}

#[derive(Debug, Clone)]
pub struct Complex {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct ComplexCollection {
    pub possibilities: Vec<Complex>,
}

#[derive(Debug)]
pub struct Operation<'a> {
    pub name: String,
    pub collection: ComplexCollection,
    pub variables: Option<Vec<variable::Variable<'a>>>,
}

impl<'a, 'b> Operation<'a> {
    pub fn compile(
        definition: &'a parsed_query::Definition,
        schema: &'b schema::Schema,
        imported_fragments: ImportedFragments,
    ) -> Result<Self> {
        let context = CompileContext {
            schema,
            imported_fragments,
        };
        match definition {
            parsed_query::Definition::Operation(op_def) => build_from_operation(&context, op_def),
            parsed_query::Definition::Fragment(frag_def) => {
                let parsed_query::TypeCondition::On(type_name) = &frag_def.type_condition;
                let mut parent = ComplexTraversal::try_from((&context, type_name.as_ref()))?;
                collect_fields_from_selection_set(&context, &frag_def.selection_set, &mut parent)?;
                Ok(Operation {
                    name: frag_def.name.clone(),
                    collection: parent.try_into()?,
                    variables: None,
                })
            }
        }
    }
}

fn build_from_operation<'a>(
    context: &CompileContext,
    operation: &'a parsed_query::OperationDefinition,
) -> Result<Operation<'a>> {
    let (op_type_name, op_name, selection_set, var_defs) = match operation {
        parsed_query::OperationDefinition::Query(query) => (
            "Query",
            &query.name,
            &query.selection_set,
            &query.variable_definitions,
        ),
        parsed_query::OperationDefinition::Mutation(mutation) => (
            "Mutation",
            &mutation.name,
            &mutation.selection_set,
            &mutation.variable_definitions,
        ),
        _ => return Err(Error::OperationUnsupported),
    };
    let mut parent = ComplexTraversal::try_from((context, op_type_name))?;
    collect_fields_from_selection_set(context, selection_set, &mut parent)?;
    Ok(Operation {
        name: op_name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| op_type_name.to_string()),
        collection: parent.try_into()?,
        variables: variable::try_build_variable_ir(var_defs).map_err(Error::VariableError)?,
    })
}

fn get_type_ir_for_field(
    field_type: &schema_field::ConcreteFieldType,
    sub_traversal: FieldTraversal,
) -> Result<FieldType> {
    let field_type_ir = match &field_type.definition {
        schema_field::FieldTypeDefinition::InputObject => {
            return Err(Error::InputObjectOnSelection)
        }
        schema_field::FieldTypeDefinition::TypeName => FieldType::TypeName,
        schema_field::FieldTypeDefinition::Scalar(scalar) => FieldType::Scalar(scalar.clone()),
        schema_field::FieldTypeDefinition::Enum => FieldType::Enum(field_type.name.clone()),
        schema_field::FieldTypeDefinition::Interface
        | schema_field::FieldTypeDefinition::Union
        | schema_field::FieldTypeDefinition::Object => match sub_traversal {
            FieldTraversal::Terminal(terminal) => {
                return Err(Error::UnexpectedComplexTravseral(
                    terminal.type_name.to_string(),
                ));
            }
            FieldTraversal::Complex(complex) => FieldType::Complex(complex.try_into()?),
        },
    };
    Ok(field_type_ir)
}

fn insert_field<'a>(
    context: &'a CompileContext,
    selection_field: &'a parsed_query::Field,
    traversal: &mut ComplexTraversal<'a>,
) -> Result<()> {
    let name = &selection_field.name;
    let alias = selection_field.alias.as_ref().unwrap_or(name);
    let field = &traversal
        .fields_lookup
        .get(name)
        .ok_or_else(|| Error::UnknownField(traversal.type_name.to_string(), name.to_string()))?;
    let has_no_sub_selections = selection_field.selection_set.items.is_empty();
    let is_complex = field.type_description.is_complex();
    let name = &field.type_description.reveal_concrete().name[..];
    match (has_no_sub_selections, is_complex) {
        (true, true) => Err(Error::MissingSelectionSetOnType(name.to_string())),
        (false, false) => Err(Error::SelectionSetOnWrongType(name.to_string())),
        (true, false) => {
            let terminal = TerminalTraversal::from(name);
            traversal.insert_terminal(alias, name, field, terminal);
            Ok(())
        }
        (false, true) => {
            let mut sub_parent = ComplexTraversal::try_from((context, name))?;
            collect_fields_from_selection_set(
                context,
                &selection_field.selection_set,
                &mut sub_parent,
            )?;
            traversal.insert_complex(alias, name, field, sub_parent);
            Ok(())
        }
    }
}

fn collect_fields_from_selection_set<'a>(
    context: &'a CompileContext,
    selection_set: &'a parsed_query::SelectionSet,
    complex_parent: &mut ComplexTraversal<'a>,
) -> Result<()> {
    for selection in &selection_set.items {
        let (spread_type_name, sub_selection_set) = match selection {
            parsed_query::Selection::Field(selection_field) => {
                insert_field(context, &selection_field, complex_parent)?;
                continue;
            }
            parsed_query::Selection::InlineFragment(fragment_def) => {
                let parsed_query::TypeCondition::On(type_name) = fragment_def
                    .type_condition
                    .as_ref()
                    .ok_or_else(|| Error::MissingTypeCondition)?;
                (type_name, &fragment_def.selection_set)
            }
            parsed_query::Selection::FragmentSpread(spread) => {
                let fragment_def = context
                    .imported_fragments
                    .get(&spread.fragment_name)
                    .ok_or_else(|| Error::UnknownFragment(spread.fragment_name.clone()))?;
                let parsed_query::TypeCondition::On(type_name) = &fragment_def.type_condition;
                (type_name, &fragment_def.selection_set)
            }
        };
        let mut sub_parent = complex_parent.clone_for_type_spread(context, spread_type_name)?;
        collect_fields_from_selection_set(context, sub_selection_set, &mut sub_parent)?;
        complex_parent.extend_from(sub_parent);
    }
    Ok(())
}
