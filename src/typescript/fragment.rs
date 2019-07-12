use super::complex::from_selection_set;
use super::{CompileContext, Parent, Result, Typescript};
use graphql_parser::query::{FragmentDefinition, TypeCondition};

pub fn from_fragment(
    ctx: &mut CompileContext,
    fragment: &FragmentDefinition,
) -> Result<(String, Typescript)> {
    let name = fragment.name.clone();
    let TypeCondition::On(type_name) = &fragment.type_condition;
    let parent = Parent {
        type_name: type_name.to_string(),
        compiled_name: name.clone(),
    };
    let type_defs = from_selection_set(ctx, &fragment.selection_set, &parent)?;
    let imports = ctx.compile_imports();
    let contents = format!("{}{}", imports, type_defs.join("\n\n"));
    Ok((name, contents))
}
