use crate::graphql::schema::field::ScalarType;
use crate::graphql::{BottomTypeConfig, CompileConfig};

pub fn compile_scalar(config: &CompileConfig, scalar: &ScalarType) -> String {
    match scalar {
        ScalarType::Boolean => String::from("boolean"),
        ScalarType::String | ScalarType::ID => String::from("string"),
        ScalarType::Float | ScalarType::Int => String::from("number"),
        ScalarType::Custom(name) => match &config.bottom_type_config {
            BottomTypeConfig::UseBottomType => String::from("any"),
            BottomTypeConfig::UseRealName => name.clone(),
            BottomTypeConfig::UseRealNameWithPrefix(s) => format!("{}{}", s, name),
        },
    }
}
