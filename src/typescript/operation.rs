use super::complex::from_selection_set;
use super::variable::from_variable_defs;
use super::{CompileContext, Error, Parent, Result, Typescript};
use graphql_parser::query::OperationDefinition;

pub fn from_operation(
    ctx: &mut CompileContext,
    operation: &OperationDefinition,
) -> Result<(String, Typescript)> {
    let (type_name, op_name, selection_set, var_defs) = match operation {
        OperationDefinition::Query(query) => (
            "Query",
            &query.name,
            &query.selection_set,
            &query.variable_definitions,
        ),
        OperationDefinition::Mutation(mutation) => (
            "Mutation",
            &mutation.name,
            &mutation.selection_set,
            &mutation.variable_definitions,
        ),
        _ => return Err(Error::OperationUnsupported),
    };
    let compiled_name = op_name.clone().unwrap_or_else(|| type_name.to_string());
    let parent = Parent {
        type_name: type_name.to_string(),
        compiled_name,
    };
    let mut type_defs = from_selection_set(ctx, selection_set, &parent)?;
    if let Some(mut var_defs) = from_variable_defs(ctx, var_defs, &parent.compiled_name)?.as_mut() {
        type_defs.append(&mut var_defs);
    }
    let imports = ctx.compile_imports();
    let contents = format!("{}{}", imports, type_defs.join("\n\n"));
    Ok((parent.compiled_name, contents))
}
