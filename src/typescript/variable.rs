use super::{CompileContext, Result};
use graphql_parser::query;

fn from_variable_graph_name(ctx: &mut CompileContext, name: &str) -> Result<String> {
    let name_ref = match name {
        "ID" | "String" => "string",
        "Float" | "Int" => "number",
        "Boolean" => "boolean",
        _ => {
            ctx.add_type(name);
            name
        }
    };
    Ok(String::from(name_ref))
}

fn from_variable_type(
    ctx: &mut CompileContext,
    var_type: &query::Type,
    is_nullable: bool,
) -> Result<String> {
    let output = match var_type {
        query::Type::NamedType(graph_name) => {
            let name = from_variable_graph_name(ctx, graph_name)?;
            if is_nullable {
                return Ok(format!("{} | null", name));
            }
            name
        }
        query::Type::ListType(inner) => {
            let inner_str = from_variable_type(ctx, inner, true)?;
            let type_str = format!("({})[]", inner_str);
            if is_nullable {
                return Ok(format!("{} | null", type_str));
            }
            type_str
        }
        query::Type::NonNullType(inner) => from_variable_type(ctx, inner, false)?,
    };
    Ok(output)
}

pub fn from_variable_defs(
    ctx: &mut CompileContext,
    defs: &[query::VariableDefinition],
    parent_name: &str,
) -> Result<Option<Vec<String>>> {
    if defs.is_empty() {
        return Ok(None);
    }
    let mut fields = Vec::with_capacity(defs.len());
    let mut types = Vec::new();
    for def in defs {
        let type_name = from_variable_type(ctx, &def.var_type, true)?;
        let prop_line = if let query::Type::NonNullType(_) = def.var_type {
            format!("  {}: {};", def.name, type_name)
        } else {
            format!("  {}?: {};", def.name, type_name)
        };
        fields.push(prop_line);
    }
    let variables_type = format!(
        "export interface {}Variables {{\n{}\n}}",
        parent_name,
        fields.join("\n")
    );
    types.push(variables_type);
    Ok(Some(types))
}
