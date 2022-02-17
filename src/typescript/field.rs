use crate::graphql::schema::field::ScalarType;
use crate::graphql::{BottomTypeConfig, CompileConfig};

pub fn compile_scalar(config: &CompileConfig, scalar: &ScalarType) -> String {
    match scalar {
        ScalarType::Boolean => String::from("boolean"),
        ScalarType::String | ScalarType::Id => String::from("string"),
        ScalarType::Float | ScalarType::Int => String::from("number"),
        ScalarType::Custom(name) => match &config.bottom_type_config {
            BottomTypeConfig::DefaultBottomType => String::from("any"),
            BottomTypeConfig::RealName => name.clone(),
            BottomTypeConfig::RealNameWithPrefix(s) => format!("{s}{name}"),
        },
    }
}
