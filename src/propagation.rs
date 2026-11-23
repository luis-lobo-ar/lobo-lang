use super::*;

pub(super) fn shape(variant: &Variant) -> Option<(&Type, &Type)> {
    if variant.cases.len() != 2 {
        return None;
    }
    let ok = variant
        .cases
        .iter()
        .find(|case| case.name == "Ok")?
        .payload
        .as_ref()?;
    let error = variant
        .cases
        .iter()
        .find(|case| case.name == "Err")?
        .payload
        .as_ref()?;
    Some((ok, error))
}

pub(super) fn types(records: &records::RecordTable<'_>, ty: &Type, code: &str) -> Fallible<(Type, Type)> {
    let Type::Variant(id) = ty else {
        return Err(action_error(
            code,
            "try requires exactly Ok(value) and Err(error) variant cases",
        ));
    };
    shape(records.get_variant(id)?)
        .map(|(ok, error)| (ok.clone(), error.clone()))
        .ok_or_else(|| {
            action_error(
                code,
                "try requires exactly Ok(value) and Err(error) variant cases",
            )
        })
}

pub(super) fn present(program: &Program) -> Fallible<bool> {
    let value =
        serde_json::to_value(program).map_err(|e| action_error("internal.invalid_ir", e.to_string()))?;
    let mut remaining = vec![&value];
    while let Some(value) = remaining.pop() {
        match value {
            serde_json::Value::Object(fields) => {
                if fields.get("kind").and_then(serde_json::Value::as_str) == Some("Unary")
                    && fields.get("op").and_then(serde_json::Value::as_str) == Some("try")
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

pub(super) enum EvaluationError {
    Fault(Diagnostic),
    Return(Value),
}
impl From<Diagnostic> for EvaluationError {
    fn from(value: Diagnostic) -> Self {
        Self::Fault(value)
    }
}
impl EvaluationError {
    pub(super) fn contract_fault(self) -> Diagnostic {
        match self {
            Self::Fault(error) => error,
            Self::Return(_) => action_error("internal.invalid_ir", "try in a checked contract"),
        }
    }
}
