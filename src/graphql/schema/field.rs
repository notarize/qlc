use super::{json, Documentation, Error};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone)]
pub enum ScalarType {
    Custom(String),
    Boolean,
    String,
    Float,
    Int,
    ID,
}

impl From<&str> for ScalarType {
    fn from(name: &str) -> Self {
        match name {
            "Boolean" => ScalarType::Boolean,
            "String" => ScalarType::String,
            "Float" => ScalarType::Float,
            "Int" => ScalarType::Int,
            "ID" => ScalarType::ID,
            _ => ScalarType::Custom(name.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum FieldTypeDefinition {
    Object,
    Interface,
    Union,
    Enum,
    Scalar(ScalarType),
    InputObject,
    TypeName,
}

impl FieldTypeDefinition {
    fn is_complex(&self) -> bool {
        match self {
            Self::Union | Self::Object | Self::Interface | Self::InputObject => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FieldTypeModifier {
    /// No modifier or "flat"
    None,
    /// Type can be null
    Nullable,
    /// List of type
    List,
    /// List or type that can itself be null
    NullableList,
    /// Non-nullable list of type or nulls
    ListOfNullable,
    /// List of type or nulls that can itself be null
    NullableListOfNullable,
}

impl FieldTypeModifier {
    fn new() -> Self {
        FieldTypeModifier::Nullable
    }
}

#[derive(Debug)]
pub struct ConcreteFieldType {
    pub name: String,
    pub modifier: FieldTypeModifier,
    pub definition: FieldTypeDefinition,
}

struct ModifierBuilder {
    concrete: FieldTypeModifier,
    higher_order_modifiers: Vec<FieldTypeModifier>,
}

impl ModifierBuilder {
    fn new() -> Self {
        ModifierBuilder {
            concrete: FieldTypeModifier::new(),
            higher_order_modifiers: Vec::new(),
        }
    }

    /// Transition one level less "nullable"
    fn actualize(&mut self) {
        self.concrete = match &self.concrete {
            FieldTypeModifier::Nullable => FieldTypeModifier::None,
            FieldTypeModifier::NullableListOfNullable => FieldTypeModifier::NullableList,
            FieldTypeModifier::ListOfNullable => FieldTypeModifier::List,
            old_concrete => {
                self.higher_order_modifiers.push(old_concrete.clone());
                FieldTypeModifier::new()
            }
        };
    }

    /// Transition into a list version
    fn listize(&mut self) {
        self.concrete = match &self.concrete {
            FieldTypeModifier::Nullable => FieldTypeModifier::NullableListOfNullable,
            FieldTypeModifier::None => FieldTypeModifier::ListOfNullable,
            old_concrete => {
                self.higher_order_modifiers.push(old_concrete.clone());
                FieldTypeModifier::new()
            }
        };
    }
}

#[derive(Debug)]
pub struct FieldType {
    concrete: ConcreteFieldType,
    higher_order_modifiers: Vec<FieldTypeModifier>,
}

impl FieldType {
    fn new_type_name() -> Self {
        let concrete = ConcreteFieldType {
            name: "__typename".to_string(),
            modifier: FieldTypeModifier::None,
            definition: FieldTypeDefinition::TypeName,
        };
        FieldType {
            concrete,
            higher_order_modifiers: Vec::new(),
        }
    }

    pub fn type_modifier_iter(&self) -> impl Iterator<Item = &FieldTypeModifier> {
        self.higher_order_modifiers
            .iter()
            .chain(std::iter::once(&self.concrete.modifier))
    }

    pub fn reveal_concrete(&self) -> &ConcreteFieldType {
        &self.concrete
    }

    pub fn is_complex(&self) -> bool {
        self.concrete.definition.is_complex()
    }
}

impl TryFrom<json::FieldType> for FieldType {
    type Error = Error;
    fn try_from(json: json::FieldType) -> Result<Self, Self::Error> {
        let mut modifier_builder = ModifierBuilder::new();
        let mut iter = json;
        loop {
            let kind = iter.kind.as_ref();
            let name = match kind {
                "NON_NULL" => {
                    iter = *iter.of_type.ok_or_else(|| Error::MissingTypeOfForNonNull)?;
                    modifier_builder.actualize();
                    continue;
                }
                "LIST" => {
                    iter = *iter.of_type.ok_or_else(|| Error::MissingTypeOfForList)?;
                    modifier_builder.listize();
                    continue;
                }
                _ => iter.name.ok_or_else(|| Error::MissingNameForField)?,
            };
            let definition = match kind {
                "OBJECT" => FieldTypeDefinition::Object,
                "SCALAR" => FieldTypeDefinition::Scalar(ScalarType::from(name.as_ref())),
                "INTERFACE" => FieldTypeDefinition::Interface,
                "ENUM" => FieldTypeDefinition::Enum,
                "INPUT_OBJECT" => FieldTypeDefinition::InputObject,
                "UNION" => FieldTypeDefinition::Union,
                _ => {
                    return Err(Error::UnknownType {
                        name,
                        kind: iter.kind,
                    })
                }
            };
            let ModifierBuilder {
                higher_order_modifiers,
                concrete,
            } = modifier_builder;
            let real_field_type = ConcreteFieldType {
                definition,
                name,
                modifier: concrete,
            };
            return Ok(FieldType {
                concrete: real_field_type,
                higher_order_modifiers,
            });
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub documentation: Documentation,
    pub type_description: FieldType,
}

impl Field {
    pub(super) fn new_type_name() -> Self {
        Field {
            name: "__typename".to_string(),
            documentation: None,
            type_description: FieldType::new_type_name(),
        }
    }
}

impl TryFrom<json::Field> for Field {
    type Error = Error;
    fn try_from(json: json::Field) -> Result<Self, Error> {
        let json::Field {
            type_information,
            name,
            description,
        } = json;
        Ok(Field {
            name,
            documentation: description.map(|docs| {
                docs.lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            }),
            type_description: type_information.try_into()?,
        })
    }
}
