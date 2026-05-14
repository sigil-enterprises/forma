use serde_json::Value;

/// Extract a string field from a JSON object, returning empty string if missing/null.
fn str_field(obj: &Value, key: &str) -> String {
    obj.as_object()
        .and_then(|o| o.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Get first element of array where attr matches value.
fn first_by_attr(arr: &Value, attr: &str, value: &str) -> Value {
    if let Some(items) = arr.as_array() {
        for item in items {
            if let Some(obj) = item.as_object() {
                if let Some(attr_val) = obj.get(attr) {
                    if attr_val.as_str() == Some(value) {
                        return item.clone();
                    }
                }
            }
        }
    }
    Value::Null
}

/// Extract array of string values from an attribute of matching items.
fn map_attr(arr: &Value, attr: &str, match_attr: &str, match_val: &str) -> Vec<Value> {
    if let Some(items) = arr.as_array() {
        items
            .iter()
            .filter_map(|item| {
                if let Some(obj) = item.as_object() {
                    if let Some(attr_v) = obj.get(match_attr) {
                        if attr_v.as_str() == Some(match_val) {
                            return obj.get(attr).cloned();
                        }
                    }
                }
                None
            })
            .collect()
    } else {
        vec![]
    }
}

/// Recursively rebuild a Value through fresh serde_json construction.
fn rebuild_value(v: &Value) -> Value {
    match v {
        Value::Null => Value::Null,
        Value::Bool(b) => Value::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() { return Value::Number(i.into()); }
            if let Some(u) = n.as_u64() { return Value::Number(u.into()); }
            if let Some(f) = n.as_f64() {
                return serde_json::Number::from_f64(f).map(Value::Number).unwrap_or_else(|| Value::Number(n.clone()));
            }
            Value::Number(n.clone())
        },
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => Value::Array(arr.iter().map(rebuild_value).collect()),
        Value::Object(obj) => {
            let mut new = serde_json::Map::new();
            for (k, val) in obj {
                new.insert(k.clone(), rebuild_value(val));
            }
            Value::Object(new)
        },
    }
}

/// Normalize a slides/sections array so every item has standard template keys.
/// Tera strict mode throws on missing properties — Jinja2 returns Undefined.
/// Adding empty-string defaults keeps templates compatible across both engines.
fn normalize_items(items: &Value) -> Value {
    if let Some(arr) = items.as_array() {
        Value::Array(arr.iter().map(|item| {
            let mut map = item.as_object().cloned().unwrap_or_default();
            if !map.contains_key("title") {
                map.insert("title".to_string(), Value::String("".to_string()));
            }
            if !map.contains_key("content") {
                map.insert("content".to_string(), Value::String("".to_string()));
            }
            if !map.contains_key("subtitle") {
                map.insert("subtitle".to_string(), Value::String("".to_string()));
            }
            Value::Object(map)
        }).collect())
    } else {
        Value::Null
    }
}

pub fn build_context(document: &Value, style: &Value) -> Value {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let forma_version = env!("CARGO_PKG_VERSION");

    // Normalize slides/sections so every item has standard template keys.
    // Tera strict mode throws on missing properties — Jinja2 returns Undefined.
    let mut doc = document.as_object().cloned().unwrap_or_default();
    if let Some(slides) = doc.get("slides") {
        doc.insert("slides".to_string(), normalize_items(slides));
    }
    if let Some(sections) = doc.get("sections") {
        doc.insert("sections".to_string(), normalize_items(sections));
    }
    let doc_normalized = Value::Object(doc);

    // Pre-compute common complex expressions that Jinja2 templates do via filter chains.
    // Tera 1.20 does not support string-literal filter arguments, so these are computed here.
    let slides = doc_normalized
        .as_object()
        .and_then(|o| o.get("slides"))
        .unwrap_or(&Value::Null);

    // Cover slide (type == "cover")
    let cover = first_by_attr(slides, "type", "cover");
    let cover_client = str_field(&cover, "client");
    let cover_title = str_field(&cover, "title");

    // slide index tracking variable
    let slide_count = slides.as_array().map_or(0, |a| a.len());

    // Compute per-type slide lists (replaces selectattr | map chains)
    let cover_slides = map_attr(slides, "client", "type", "cover");
    let cover_title_list = map_attr(slides, "title", "type", "cover");

    let page = serde_json::json!({
        "cover_client": cover_client,
        "cover_title": cover_title,
        "cover_slides": cover_slides,
        "cover_titles": cover_title_list,
        "slide_count": slide_count,
        "default_currency": "USD",
    });

    // Wrap in [0] accessor pattern: page[0] → document, page[0].slides → slides
    let page_accessor = serde_json::json!([doc_normalized.clone()]);

    let context = serde_json::json!({
        "document": doc_normalized,
        "content": doc_normalized,
        "style": style,
        "meta": {
            "rendered_date": today,
            "forma_version": forma_version,
            "project_dir": "",
            "presskit_root": "",
        },
        "page": page,
        "page_accessor": page_accessor,
    });

    rebuild_value(&context)
}
