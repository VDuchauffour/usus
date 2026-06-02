// Response parser: handles JSON and `;0x<hex>;<js-expr>` responses.
//
// The opencode.ai server function endpoint returns either plain JSON or a
// TanStack-Start serialized JS snippet (`;0x<hex>;<expr>`) that needs JS
// evaluation. `parse_js_response` handles the latter via an embedded JS engine.

use anyhow::{Context as _, Result, anyhow};
use boa_engine::{Context as JsContext, Source};
use serde_json::Value;

/// Strip the `;0x[hex];` prefix used by the JS-style payload.
fn strip_js_prefix(text: &str) -> &str {
    let Some(rest) = text.strip_prefix(";0x") else {
        return text;
    };
    let Some(idx) = rest.find(';') else {
        return text;
    };
    if rest[..idx].chars().all(|c| c.is_ascii_hexdigit()) {
        &rest[idx + 1..]
    } else {
        text
    }
}

/// The JS payload ends with `($R["server-fn:0"]))` referencing a bare `$R`
/// while assignments use `self.$R`. Rewrite the trailing expression so the
/// embedded engine resolves it through the same object.
fn fix_server_fn(s: &str) -> String {
    let trimmed = s.trim_end();
    let needle = r#"($R["server-fn:0"]))"#;
    let replacement = r#"(self.$R["server-fn:0"]))"#;
    if let Some(prefix) = trimmed.strip_suffix(needle) {
        format!("{prefix}{replacement}")
    } else {
        trimmed.to_string()
    }
}

fn parse_js_response(text: &str) -> Result<Value> {
    let stripped = strip_js_prefix(text);
    let fixed = fix_server_fn(stripped);

    let mut ctx = JsContext::default();
    ctx.eval(Source::from_bytes(
        b"globalThis.self = {}; globalThis.$R = []; self.$R = globalThis.$R;",
    ))
    .map_err(|e| anyhow!("JS bootstrap failed: {e}"))?;

    // Evaluate the response expression and capture its result.
    let prog = format!("globalThis.__result = ({fixed});");
    ctx.eval(Source::from_bytes(prog.as_bytes()))
        .map_err(|e| anyhow!("JS eval failed: {e}"))?;

    let stringified = ctx
        .eval(Source::from_bytes(b"JSON.stringify(globalThis.__result)"))
        .map_err(|e| anyhow!("JSON.stringify failed: {e}"))?;
    let s = stringified
        .to_string(&mut ctx)
        .map_err(|e| anyhow!("JSON.stringify result not stringifiable: {e}"))?
        .to_std_string()
        .map_err(|e| anyhow!("Non-UTF8 JSON: {e}"))?;
    serde_json::from_str(&s).context("Parsing eval'd JSON")
}

fn unwrap_value(v: Value) -> Value {
    if let Value::Array(arr) = &v
        && arr.len() == 1
    {
        return arr[0].clone();
    }
    if let Value::Object(map) = &v {
        if let Some(inner) = map.get("value") {
            return inner.clone();
        }
        if let Some(inner) = map.get("_$value") {
            return inner.clone();
        }
    }
    v
}

pub fn extract_data(text: &str) -> Result<(Vec<Value>, Vec<Value>)> {
    let trimmed = text.trim_start();
    let parsed = if trimmed.contains("text/javascript") || trimmed.starts_with(";0x") {
        parse_js_response(trimmed)?
    } else {
        serde_json::from_str(trimmed).context("Parsing JSON response")?
    };

    let unwrapped = unwrap_value(parsed);

    let usage = match unwrapped.get("usage") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => match &unwrapped {
            Value::Array(arr) => arr.clone(),
            _ => Vec::new(),
        },
    };
    let keys = unwrapped
        .get("keys")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok((usage, keys))
}
