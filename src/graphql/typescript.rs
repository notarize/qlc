use super::document::{
    DocumentDefinition, NamedType, OperationDefinition, ProductTypeDefinition, TypeDefinition,
};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    FileError(std::io::Error),
}

fn make_name_str(product_name: &str, field_name: &str, named_type: &NamedType) -> String {
    match named_type {
        NamedType::UserDefined => format!("{}_{}", product_name, field_name),
        NamedType::Null => String::from("null"),
        NamedType::Boolean => String::from("boolean"),
        NamedType::String | NamedType::ID => String::from("string"),
        NamedType::Float | NamedType::Int => String::from("number"),
        NamedType::Unknown => String::from("any"),
        NamedType::List(inner_names) => {
            let inner_str = join_sum_type_str(product_name, field_name, inner_names.iter());
            format!("({})[]", inner_str)
        }
    }
}

fn join_sum_type_str<'a, I>(product_name: &str, field_name: &str, names: I) -> String
where
    I: Iterator<Item = &'a NamedType>,
{
    names
        .map(|named| make_name_str(product_name, field_name, named))
        .collect::<Vec<String>>()
        .join(" | ")
}

fn make_product_str(product_name: &str, prod_def: &ProductTypeDefinition) -> String {
    prod_def
        .fields
        .iter()
        .map(|field| {
            let field_name = &field.name;
            let type_def = join_sum_type_str(product_name, field_name, field.field_type.names.iter());
            format!("  {}: {};", field_name, type_def)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn make_str(op: &OperationDefinition) -> String {
    op.type_defs
        .iter()
        .map(|type_def| match type_def {
            TypeDefinition::Product(prod_def) => {
                let name = &prod_def.name;
                let inner = make_product_str(&name, prod_def);
                format!("export interface {} {{\n{}\n}}", name, inner)
            }
            TypeDefinition::Sum(sum_def) => {
                let name = "hi";
                format!("export type {} {{\n{}\n}}", name, name)
            }
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn makedir_p(path: &PathBuf) -> Result<(), Error> {
    match std::fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(Error::FileError(e)),
    }
}

pub fn write(doc: &DocumentDefinition, file_path: &PathBuf) -> Result<(), Error> {
    let mut generated_dir_path = file_path.clone();
    generated_dir_path.pop();
    generated_dir_path.push("__generated__");
    makedir_p(&generated_dir_path)?;

    for op in &doc.definitions {
        let file_contents = make_str(&op);
        generated_dir_path.push(format!("{}.ts", op.name));;
        std::fs::write(&generated_dir_path, file_contents).map_err(Error::FileError)?;
        generated_dir_path.pop();
    }

    Ok(())
}
