use super::*;

// Check every serialized type position, including annotations, generic arguments and
// compound values, as well as references whose type comes from their named target.
pub(super) fn present(program: &Program) -> Fallible<bool> {
    let actionful: BTreeSet<_> = program
        .functions
        .iter()
        .filter(|f| !f.actions.is_empty())
        .map(|f| f.id.as_str())
        .collect();
    let value =
        serde_json::to_value(program).map_err(|e| action_error("internal.invalid_ir", e.to_string()))?;
    let mut remaining = vec![&value];
    while let Some(value) = remaining.pop() {
        match value {
            serde_json::Value::Object(fields) => {
                let typed = fields.get("Function").or_else(|| fields.get("signature"));
                if typed
                    .and_then(|s| s.get("actions"))
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|a| !a.is_empty())
                {
                    return Ok(true);
                }
                if fields.get("kind").and_then(serde_json::Value::as_str) == Some("Reference")
                    && fields
                        .get("function")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|id| actionful.contains(id))
                {
                    return Ok(true);
                }
                remaining.extend(fields.values());
            }
            serde_json::Value::Array(values) => remaining.extend(values),
            _ => {}
        }
    }
    Ok(false)
}
