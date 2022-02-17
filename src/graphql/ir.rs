use super::ParsedTextType;
use crate::cli::{similar_help_suggestions, PrintableMessage};
use crate::graphql::schema;
use crate::graphql::schema::field as schema_field;
use crate::graphql::variable;
use graphql_parser::query as parsed_query;
use graphql_parser::Pos;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::path::Path;

#[derive(Debug)]
pub enum Warning {
    OverFragmentNarrowing {
        position: Pos,
        possible_types: Vec<String>,
        spread_type_name: String,
    },
    DeprecatedFieldUse {
        position: Pos,
        field_name: String,
        parent_type_name: String,
    },
}

impl From<(&str, &Path, Warning)> for PrintableMessage {
    fn from((contents, file_path, warning): (&str, &Path, Warning)) -> Self {
        match warning {
            Warning::OverFragmentNarrowing {
                position,
                possible_types,
                spread_type_name,
            } => PrintableMessage::new_compile_warning(
                &format!("fragment over narrowing with type `{spread_type_name}`"),
                file_path,
                contents,
                &position,
                Some(&format!(
                    "The parent types of this spread are limited to `{}`, making spreading `{spread_type_name}` uneeded.",
                    possible_types.join("`, `"),
                )),
            ),
            Warning::DeprecatedFieldUse { position, field_name, parent_type_name } => PrintableMessage::new_compile_warning(
                &format!("use of deprecated field `{field_name}` on type `{parent_type_name}`"),
                file_path,
                contents,
                &position,
                None
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ForeignFragmentJumpState {
    RootLocal,
    JustChanged,
    FullyJumped,
}

impl ForeignFragmentJumpState {
    fn is_fully_jumped(&self) -> bool {
        matches!(self, ForeignFragmentJumpState::FullyJumped)
    }

    fn is_local(&self) -> bool {
        matches!(self, ForeignFragmentJumpState::RootLocal)
    }

    fn jump_inline_one_level(&self) -> Self {
        if self.is_local() {
            ForeignFragmentJumpState::RootLocal
        } else {
            ForeignFragmentJumpState::FullyJumped
        }
    }

    fn jump_foreigin_one_level(&self) -> Self {
        if self.is_local() {
            ForeignFragmentJumpState::JustChanged
        } else {
            ForeignFragmentJumpState::FullyJumped
        }
    }
}

#[derive(Debug)]
pub enum Error {
    SelectionSetAsOperationUnsupported(Pos),
    UnknownFragment(String, Pos, Vec<String>),
    MissingTypeConditionOnInlineFragment(Pos),
    SelectionSetOnWrongType(String, Pos),
    MissingSelectionSetOnType(String, Pos),
    UnknownField {
        parent_type_name: String,
        field_name: String,
        position: Pos,
        possible_field_names: Vec<String>,
    },
    Variable(variable::Error),
    MissingType(String),
    UnexpectedComplexTravseral(String),
    InputObjectOnSelection {
        field_name: String,
        type_name: String,
    },
    MixedTerminalAndComplexFields {
        terminal_type_name: String,
        complex_type_name: String,
    },
}

impl From<(&str, &Path, Error)> for PrintableMessage {
    fn from((contents, file_path, error): (&str, &Path, Error)) -> Self {
        match error {
            Error::SelectionSetAsOperationUnsupported(position) => {
                PrintableMessage::new_compile_error(
                    "unsupported selection set as operation",
                    file_path,
                    contents,
                    &position,
                    Some("QLC does not support a plain selection set as an operation."),
                )
            }
            Error::UnknownFragment(name, position, possible_spread_names) => {
                let extra = similar_help_suggestions(&name, possible_spread_names.into_iter())
                    .unwrap_or_else(|| " Did you forget to import it?".to_string());
                PrintableMessage::new_compile_error(
                    &format!("unknown spread fragment name `{name}`"),
                    file_path,
                    contents,
                    &position,
                    Some(&format!("This fragment name doesn't appear to be in scope.{extra}")),
                )
            }
            Error::MissingTypeConditionOnInlineFragment(position) => PrintableMessage::new_compile_error(
                "fragment missing type condition on inline fragment",
                file_path,
                contents,
                &position,
                Some("Fragments must specify a type they can be spread on."),
            ),
            Error::SelectionSetOnWrongType(name, position) => PrintableMessage::new_compile_error(
                &format!("unexpected selection on field of type `{name}`"),
                file_path,
                contents,
                &position,
                Some("This field is not a complex type with selections. Did you accidentally place the curlies on this field?"),
            ),
            Error::MissingSelectionSetOnType(name, position) => {
                PrintableMessage::new_compile_error(
                    &format!("expected selection on field of type `{name}`"),
                    file_path,
                    contents,
                    &position,
                    Some("This is a complex type, and it is improper GraphQL to not have at least one sub field selection."),
                )
            }
            Error::UnknownField {
                parent_type_name,
                field_name,
                position,
                possible_field_names,
            } => {
                let extra = similar_help_suggestions(&field_name, possible_field_names.into_iter()).unwrap_or_else(String::new);
                PrintableMessage::new_compile_error(
                    &format!("unknown field `{field_name}`"),
                    file_path,
                    contents,
                    &position,
                    Some(&format!("Check the fields of `{parent_type_name}`.{extra}")),
                )
            }
            Error::Variable(var_error) => {
                PrintableMessage::from((contents, file_path, var_error))
            }
            Error::MissingType(type_name) => PrintableMessage::new_simple_program_error(
                &format!("failed lookup of type `{type_name}`"),
            ),
            Error::InputObjectOnSelection { type_name, field_name } => {
                PrintableMessage::new_simple_program_error(
                    &format!("unexpectedly traversing field `{field_name}` with input object type `{type_name}`")
                )
            }
            Error::UnexpectedComplexTravseral(type_name) => {
                PrintableMessage::new_simple_program_error(
                    &format!("unexpectedly traversing a terminal of type `{type_name}`"),
                )
            }
            Error::MixedTerminalAndComplexFields { complex_type_name, terminal_type_name } => {
                PrintableMessage::new_simple_program_error(
                    &format!("unexpectedly attempting merge of complex type `{complex_type_name}` and terminal type `{terminal_type_name}`."),
                )
            }
        }
    }
}

type Result<T> = std::result::Result<T, Error>;
type ResultMany<T> = std::result::Result<T, Vec<Error>>;
type OperationResult<'a> =
    std::result::Result<(Operation<'a>, Vec<Warning>), (Vec<Error>, Vec<Warning>)>;
type ImportedFragments<'a> = HashMap<String, parsed_query::FragmentDefinition<'a, ParsedTextType>>;

pub struct CompileContext<'a, 'b> {
    pub schema: &'a schema::Schema,
    show_deprecation_warnings: bool,
    imported_fragments: ImportedFragments<'b>,
    warnings: std::cell::RefCell<Vec<Warning>>,
}

impl<'a, 'b> CompileContext<'a, 'b> {
    fn push_warning(&self, warning: Warning) {
        self.warnings.borrow_mut().push(warning);
    }
}

// For a few conversions with ?
impl From<Error> for Vec<Error> {
    fn from(error: Error) -> Self {
        vec![error]
    }
}

/// Alias and field name
type FieldId<'a> = (&'a str, &'a str);

#[derive(Debug, Clone)]
struct UniqueFields<'a> {
    collection: HashMap<FieldId<'a>, (&'a schema_field::Field, FieldTraversal<'a>)>,
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
    ) -> Result<()> {
        match self.collection.entry((alias, name)) {
            Entry::Occupied(occupant) => {
                occupant.into_mut().1.extend_from(traversal)?;
            }
            Entry::Vacant(vacany) => {
                vacany.insert((field, traversal));
            }
        }
        Ok(())
    }

    fn extend_from(&mut self, other: UniqueFields<'a>) -> Result<()> {
        for ((alias, name), (field, traversal)) in other.collection.into_iter() {
            self.insert(alias, name, field, traversal)?;
        }
        Ok(())
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
                    last_type_modifier: field.type_description.type_modifiers().1.clone(),
                    type_ir: get_type_ir_for_field(field, concrete, sub_traversal)?,
                })
            })
            .collect::<Result<Self>>()
            .map(|mut field_irs| {
                // For test and signature stability
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

impl<'a, 'b> ComplexTraversal<'a> {
    fn clone_for_type_spread(
        &self,
        context: &'a CompileContext<'a, 'b>,
        spread_type_name: &'a str,
        position: Pos,
        jump_state: ForeignFragmentJumpState,
    ) -> Result<Self> {
        Self::try_from((context, spread_type_name, position)).map(|mut base| {
            base.concrete_objects
                .retain(|type_name, _| self.concrete_objects.contains_key(type_name));
            if !jump_state.is_fully_jumped() && base.concrete_objects.is_empty() {
                context.push_warning(Warning::OverFragmentNarrowing {
                    position,
                    possible_types: self
                        .concrete_objects
                        .keys()
                        .map(|key| key.to_string())
                        .collect(),
                    spread_type_name: spread_type_name.to_string(),
                });
            }
            base
        })
    }

    fn insert_terminal(
        &mut self,
        alias: &'a str,
        name: &'a str,
        field: &'a schema_field::Field,
        terminal: TerminalTraversal<'a>,
    ) -> Result<()> {
        for uniques in self.concrete_objects.values_mut() {
            uniques.insert(
                alias,
                name,
                field,
                FieldTraversal::Terminal(terminal.clone()),
            )?;
        }
        Ok(())
    }

    fn insert_complex(
        &mut self,
        alias: &'a str,
        name: &'a str,
        field: &'a schema_field::Field,
        complex: ComplexTraversal<'a>,
    ) -> Result<()> {
        for uniques in self.concrete_objects.values_mut() {
            uniques.insert(alias, name, field, FieldTraversal::Complex(complex.clone()))?;
        }
        Ok(())
    }

    fn extend_from(&mut self, other: Self) -> Result<()> {
        for (type_name, other_uniques) in other.concrete_objects.into_iter() {
            if let Some(uniques) = self.concrete_objects.get_mut(type_name) {
                uniques.extend_from(other_uniques.clone())?;
            }
        }
        Ok(())
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
                // For test and signature stability
                possibilities.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                ComplexCollection { possibilities }
            })
    }
}

impl<'a, 'b> TryFrom<(&'a CompileContext<'a, 'b>, &'a str, Pos)> for ComplexTraversal<'a> {
    type Error = Error;
    fn try_from(
        (context, type_name, position): (&'a CompileContext<'a, 'b>, &'a str, Pos),
    ) -> Result<Self> {
        let schema_type = context
            .schema
            .get_type_for_name(type_name)
            .ok_or_else(|| Error::MissingType(type_name.to_string()))?;
        let fields_lookup = schema_type
            .definition
            .get_fields_lookup()
            .ok_or_else(|| Error::SelectionSetOnWrongType(type_name.to_string(), position))?;
        let concrete_objects = match &schema_type.definition {
            schema::TypeDefinition::Object(_) => {
                let mut concrete_objects = HashMap::with_capacity(1);
                concrete_objects.insert(type_name, UniqueFields::new());
                concrete_objects
            }
            schema::TypeDefinition::Interface(schema::InterfaceType { possible_types, .. })
            | schema::TypeDefinition::Union(schema::UnionType { possible_types, .. }) => {
                possible_types
                    .iter()
                    .map(|possible_type| (possible_type.as_ref(), UniqueFields::new()))
                    .collect()
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
    fn extend_from(&mut self, other: Self) -> Result<()> {
        match self {
            Self::Complex(self_complex) => match other {
                Self::Complex(other_complex) => self_complex.extend_from(other_complex),
                Self::Terminal(other_terminal) => Err(Error::MixedTerminalAndComplexFields {
                    complex_type_name: self_complex.type_name.to_string(),
                    terminal_type_name: other_terminal.type_name.to_string(),
                }),
            },
            Self::Terminal(self_terminal) => match other {
                Self::Terminal(_) => Ok(()), // Nothing to do if both terminal
                Self::Complex(other_complex) => Err(Error::MixedTerminalAndComplexFields {
                    complex_type_name: other_complex.type_name.to_string(),
                    terminal_type_name: self_terminal.type_name.to_string(),
                }),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub prop_name: String,
    pub documentation: schema::Documentation,
    pub last_type_modifier: schema_field::FieldTypeModifier,
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
        definition: &'a parsed_query::Definition<'a, ParsedTextType>,
        schema: &'b schema::Schema,
        imported_fragments: ImportedFragments<'a>,
        show_deprecation_warnings: bool,
    ) -> OperationResult<'a> {
        let context = CompileContext {
            schema,
            show_deprecation_warnings,
            imported_fragments,
            warnings: std::cell::RefCell::new(Vec::new()),
        };
        let operation = match definition {
            parsed_query::Definition::Operation(op_def) => {
                match build_from_operation(&context, op_def, ForeignFragmentJumpState::RootLocal) {
                    Ok(op) => op,
                    Err(errors) => return Err((errors, context.warnings.into_inner())),
                }
            }
            parsed_query::Definition::Fragment(frag_def) => {
                let parsed_query::TypeCondition::On(type_name) = &frag_def.type_condition;
                let mut parent = match ComplexTraversal::try_from((
                    &context,
                    type_name.as_ref(),
                    frag_def.position,
                )) {
                    Ok(p) => p,
                    Err(error) => return Err((vec![error], context.warnings.into_inner())),
                };

                if let Err(errors) = collect_fields_from_selection_set(
                    &context,
                    &frag_def.selection_set,
                    &mut parent,
                    ForeignFragmentJumpState::RootLocal,
                ) {
                    return Err((errors, context.warnings.into_inner()));
                }

                let collection = match parent.try_into() {
                    Ok(c) => c,
                    Err(error) => return Err((vec![error], context.warnings.into_inner())),
                };
                Operation {
                    name: frag_def.name.clone(),
                    collection,
                    variables: None,
                }
            }
        };
        Ok((operation, context.warnings.into_inner()))
    }
}

fn build_from_operation<'a, 'b>(
    context: &CompileContext<'a, 'b>,
    operation: &'b parsed_query::OperationDefinition<'b, ParsedTextType>,
    jump_state: ForeignFragmentJumpState,
) -> ResultMany<Operation<'b>> {
    let (op_type_name, fallback_name, op_name, selection_set, var_defs, position) = match operation
    {
        parsed_query::OperationDefinition::Query(query) => (
            "Query",
            "Query",
            &query.name,
            &query.selection_set,
            &query.variable_definitions,
            query.position,
        ),
        parsed_query::OperationDefinition::Mutation(mutation) => (
            "Mutation",
            "Mutation",
            &mutation.name,
            &mutation.selection_set,
            &mutation.variable_definitions,
            mutation.position,
        ),
        parsed_query::OperationDefinition::Subscription(subscription) => (
            "Query", // We look in query as our type for subscriptions
            "Subscription",
            &subscription.name,
            &subscription.selection_set,
            &subscription.variable_definitions,
            subscription.position,
        ),
        parsed_query::OperationDefinition::SelectionSet(selection) => {
            return Err(vec![Error::SelectionSetAsOperationUnsupported(
                selection.span.0,
            )]);
        }
    };
    let mut parent = ComplexTraversal::try_from((context, op_type_name, position))?;
    collect_fields_from_selection_set(context, selection_set, &mut parent, jump_state)?;
    Ok(Operation {
        name: op_name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| fallback_name.to_string()),
        collection: parent.try_into()?,
        variables: variable::try_build_variable_ir(context, var_defs).map_err(Error::Variable)?,
    })
}

fn get_type_ir_for_field(
    field: &schema_field::Field,
    field_type: &schema_field::ConcreteFieldType,
    sub_traversal: FieldTraversal<'_>,
) -> Result<FieldType> {
    let field_type_ir = match &field_type.definition {
        schema_field::FieldTypeDefinition::InputObject => {
            return Err(Error::InputObjectOnSelection {
                field_name: field.name.clone(),
                type_name: field_type.name.clone(),
            });
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

fn insert_field<'a, 'b>(
    context: &'a CompileContext<'a, 'b>,
    selection_field: &'a parsed_query::Field<'b, ParsedTextType>,
    traversal: &mut ComplexTraversal<'a>,
    jump_state: ForeignFragmentJumpState,
) -> ResultMany<()> {
    let name = &selection_field.name;
    let alias = selection_field.alias.as_ref().unwrap_or(name);
    let field = traversal.fields_lookup.get(name).ok_or_else(|| {
        vec![Error::UnknownField {
            parent_type_name: traversal.type_name.to_string(),
            field_name: name.to_string(),
            position: selection_field.position,
            possible_field_names: traversal.fields_lookup.keys().cloned().collect(),
        }]
    })?;
    let has_no_sub_selections = selection_field.selection_set.items.is_empty();
    let is_complex = field.type_description.is_complex();
    let field_type_name = &field.type_description.reveal_concrete().name[..];
    match (has_no_sub_selections, is_complex) {
        (true, true) => {
            return Err(vec![Error::MissingSelectionSetOnType(
                field_type_name.to_string(),
                selection_field.position,
            )])
        }
        (false, false) => {
            return Err(vec![Error::SelectionSetOnWrongType(
                field_type_name.to_string(),
                selection_field.position,
            )])
        }
        (true, false) => {
            let terminal = TerminalTraversal::from(field_type_name);
            traversal.insert_terminal(alias, field_type_name, field, terminal)?;
        }
        (false, true) => {
            let mut sub_parent =
                ComplexTraversal::try_from((context, field_type_name, selection_field.position))?;
            collect_fields_from_selection_set(
                context,
                &selection_field.selection_set,
                &mut sub_parent,
                jump_state,
            )?;
            traversal.insert_complex(alias, field_type_name, field, sub_parent)?;
        }
    };

    if context.show_deprecation_warnings && field.deprecated && jump_state.is_local() {
        context.push_warning(Warning::DeprecatedFieldUse {
            position: selection_field.position,
            field_name: field.name.to_string(),
            parent_type_name: traversal.type_name.to_string(),
        });
    }

    Ok(())
}

fn collect_fields_from_selection_set<'a, 'b>(
    context: &'a CompileContext<'a, 'b>,
    selection_set: &'a parsed_query::SelectionSet<'b, ParsedTextType>,
    complex_parent: &mut ComplexTraversal<'a>,
    jump_state: ForeignFragmentJumpState,
) -> ResultMany<()> {
    let mut errors = Vec::new();
    for selection in &selection_set.items {
        let (spread_type_name, spread_position, sub_selection_set, new_jump_state) = match selection
        {
            parsed_query::Selection::Field(selection_field) => {
                if let Err(sub_messages) =
                    insert_field(context, selection_field, complex_parent, jump_state)
                {
                    errors.extend(sub_messages);
                }
                continue;
            }
            parsed_query::Selection::InlineFragment(fragment_def) => {
                match fragment_def.type_condition {
                    Some(parsed_query::TypeCondition::On(ref type_name)) => (
                        type_name,
                        fragment_def.position,
                        &fragment_def.selection_set,
                        jump_state.jump_inline_one_level(),
                    ),
                    None => {
                        errors.push(Error::MissingTypeConditionOnInlineFragment(
                            fragment_def.position,
                        ));
                        continue;
                    }
                }
            }
            parsed_query::Selection::FragmentSpread(spread) => {
                match context.imported_fragments.get(&spread.fragment_name) {
                    Some(fragment_def) => {
                        let parsed_query::TypeCondition::On(ref type_name) =
                            fragment_def.type_condition;
                        (
                            type_name,
                            spread.position,
                            &fragment_def.selection_set,
                            jump_state.jump_foreigin_one_level(),
                        )
                    }
                    None => {
                        errors.push(Error::UnknownFragment(
                            spread.fragment_name.clone(),
                            spread.position,
                            context.imported_fragments.keys().cloned().collect(),
                        ));
                        continue;
                    }
                }
            }
        };
        // Below we only add errors when in local file. We don't want to spam duplicate messages
        // that will just be repeated when we compile the foreign fragment anyway.
        let mut sub_parent = match complex_parent.clone_for_type_spread(
            context,
            spread_type_name,
            spread_position,
            new_jump_state,
        ) {
            Ok(sp) => sp,
            Err(sub_message) => {
                if new_jump_state.is_local() {
                    errors.push(sub_message);
                }
                continue;
            }
        };
        match collect_fields_from_selection_set(
            context,
            sub_selection_set,
            &mut sub_parent,
            new_jump_state,
        ) {
            Ok(_) => {
                complex_parent.extend_from(sub_parent)?;
            }
            Err(sub_messages) => {
                if new_jump_state.is_local() {
                    errors.extend(sub_messages);
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
