use std::collections::HashMap;

use tera::{Result as TeraResult, Value};

fn latex_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(tera_latex_escape(s).into())
}

pub fn tera_latex_escape(s: &str) -> String {
    let special: &[(&str, &str)] = &[
        ("&", r"\&"),
        ("%", r"\%"),
        ("$", r"\$"),
        ("#", r"\#"),
        ("_", r"\_"),
        ("{", r"\{"),
        ("}", r"\}"),
        ("~", r"\textasciitilde{}"),
        ("^", r"\textasciicircum{}"),
        ("\\", r"\textbackslash{}"),
        ("\u{2192}", r"$\rightarrow$"),
        ("\u{2190}", r"$\leftarrow$"),
        ("\u{2013}", "--"),
        ("\u{2014}", "---"),
        ("\u{2026}", r"\ldots{}"),
        ("\u{00a0}", "~"),
    ];
    let mut result = String::new();
    let mut remaining = s;
    while !remaining.is_empty() {
        let mut best_match: Option<(&str, &str)> = None;
        let mut best_len = 0;
        for (pattern, replacement) in special {
            if remaining.starts_with(pattern) && pattern.len() > best_len {
                best_match = Some((pattern, replacement));
                best_len = pattern.len();
            }
        }
        if let Some((pattern, replacement)) = best_match {
            result.push_str(replacement);
            remaining = &remaining[pattern.len()..];
        } else {
            let c = remaining.chars().next().unwrap();
            result.push(c);
            remaining = &remaining[c.len_utf8()..];
        }
    }
    result
}

pub fn format_date_filter(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let fmt = args.get("fmt")
        .and_then(|v| v.as_str())
        .unwrap_or("%B %d, %Y");

    // Null → empty string
    if value.is_null() {
        return Ok("".into());
    }

    let date_str = if let Some(d) = value.as_str() {
        d.to_string()
    } else {
        format!("{}", value)
    };

    if date_str.is_empty() {
        return Ok("".into());
    }

    let parsed = parse_date_str(&date_str);
    match parsed {
        Some(d) => Ok(d.format(fmt).to_string().into()),
        None => Ok(date_str.into()), // return original string if unparseable
    }
}

fn parse_date_str(s: &str) -> Option<chrono::NaiveDate> {
    let patterns = ["%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%B %d, %Y"];
    for pattern in &patterns {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, pattern) {
            return Some(d);
        }
    }
    None
}

pub fn currency_filter(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let symbol = args.get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("$");
    let decimals = args.get("decimals")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as usize;

    let n: f64 = if let Some(i) = value.as_i64() {
        i as f64
    } else {
        value.as_f64().unwrap_or(0.0)
    };

    let formatted = format_decimal(n, decimals);
    Ok(format!("{}{}", symbol, formatted).into())
}

pub fn format_decimal(n: f64, decimals: usize) -> String {
    let abs = n.abs();
    let integer_part = abs as u64;
    let mut parts = Vec::new();
    let mut num = integer_part;
    if num == 0 {
        parts.push(String::from("0"));
    } else {
        while num > 0 {
            let chunk = (num % 1000) as u32;
            parts.push(format!("{:03}", chunk));
            num /= 1000;
        }
        parts.reverse();
        // Strip leading zeros from most significant chunk
        let first = parts[0].trim_start_matches('0');
        parts[0] = if first.is_empty() {
            String::from("0")
        } else {
            first.to_string()
        };
    }
    let int_str = parts.join(",");

    if decimals == 0 {
        int_str
    } else {
        let frac_str = format!("{:.decimals$}", abs, decimals = decimals);
        if let Some(dot_pos) = frac_str.find('.') {
            let frac_part = &frac_str[dot_pos + 1..];
            format!("{}.{}", int_str, frac_part)
        } else {
            int_str
        }
    }
}

pub fn join_oxford_filter(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let items = value_as_strings(value);
    let conjunction = args.get("conjunction")
        .and_then(|v| v.as_str())
        .unwrap_or("and");

    Ok(oxford_join(&items, conjunction).into())
}

pub fn value_as_strings(v: &Value) -> Vec<String> {
    if let Some(arr) = v.as_array() {
        arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()
    } else {
        Vec::new()
    }
}

pub fn oxford_join(items: &[String], conjunction: &str) -> String {
    if items.is_empty() {
        return String::new();
    }
    if items.len() == 1 {
        return items[0].clone();
    }
    if items.len() == 2 {
        return format!("{} {} {}", items[0], conjunction, items[1]);
    }
    let mut result = items[..items.len() - 1].join(", ");
    result.push_str(&format!(", {} {}", conjunction, items[items.len() - 1]));
    result
}

pub fn hex_color_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = value.as_str().unwrap_or("");
    Ok(s.trim_start_matches('#').into())
}

pub fn bullet_list_filter(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let items = value_as_strings(value);
    let indent = args.get("indent")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as usize;

    if items.is_empty() {
        return Ok("".into());
    }

    let pad = "  ".repeat(indent);
    let mut lines = vec![format!("{}\\begin{{itemize}}", pad)];
    for item in &items {
        lines.push(format!("{}  \\item {}", pad, tera_latex_escape(item)));
    }
    lines.push(format!("{}\\end{{itemize}}", pad));
    Ok(lines.join("\n").into())
}

/// Jinja2 `| default('value')` — Tera's default doesn't accept args
pub fn default_filter(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let fallback = args.get("value")
        .cloned()
        .unwrap_or_else(|| Value::String(String::new()));

    // If value is null, empty string, or missing → return fallback
    if value.is_null() {
        return Ok(fallback);
    }
    if let Some(s) = value.as_str() {
        if s.is_empty() {
            return Ok(fallback);
        }
    }
    // Non-empty, non-null → return original
    Ok(value.clone())
}

/// Jinja2 `| selectattr('attr','eq','val')` — filter array by attribute equality
pub fn selectattr_filter(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let attr = args.get("attribute")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::from("selectattr requires 'attribute' argument"))?;
    // Tera doesn't use 'eq' — it's the only comparison mode
    let _cmp = args.get("comparator")
        .and_then(|v| v.as_str())
        .unwrap_or("eq");
    let val = args.get("value")
        .ok_or_else(|| tera::Error::from("selectattr requires 'value' argument"))?;

    if let Some(arr) = value.as_array() {
        let filtered: Vec<Value> = arr.iter()
            .filter(|v| {
                if let Some(obj) = v.as_object() {
                    if let Some(attr_val) = obj.get(attr) {
                        return attr_val == val;
                    }
                }
                false
            })
            .cloned()
            .collect();
        Ok(serde_json::to_value(filtered)
            .map_err(|e| tera::Error::from(e.to_string()))?
            .into())
    } else {
        Ok(Value::Array(vec![]))
    }
}

/// Jinja2 `| map(attribute='key')` — extract attribute from objects in array
pub fn map_filter(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let attr = args.get("attribute")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::from("map requires 'attribute' argument"))?;

    if let Some(arr) = value.as_array() {
        let mapped: Vec<Value> = arr.iter()
            .filter_map(|v| {
                if let Some(obj) = v.as_object() {
                    obj.get(attr).cloned()
                } else {
                    None
                }
            })
            .collect();
        Ok(serde_json::to_value(mapped)
            .map_err(|e| tera::Error::from(e.to_string()))?
            .into())
    } else {
        Ok(Value::Array(vec![]))
    }
}

/// Jinja2 `| first` — get first element of array
pub fn first_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    if let Some(arr) = value.as_array() {
        if let Some(first) = arr.first() {
            return Ok(first.clone());
        }
    }
    Ok(Value::Null)
}

/// Jinja2 `| e` — HTML escape (alias for escape)
pub fn escape_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let s = html_escape::encode_safe(value.as_str().unwrap_or("")).to_string();
    Ok(s.into())
}

pub fn register_filters(tera: &mut tera::Tera) {
    tera.register_filter("latex_escape", latex_filter);
    tera.register_filter("le", latex_filter);
    tera.register_filter("format_date", format_date_filter);
    tera.register_filter("currency", currency_filter);
    tera.register_filter("join_oxford", join_oxford_filter);
    tera.register_filter("hex_color", hex_color_filter);
    tera.register_filter("bullet_list", bullet_list_filter);
    // Jinja2 compatibility filters
    tera.register_filter("default", default_filter);
    tera.register_filter("selectattr", selectattr_filter);
    tera.register_filter("map", map_filter);
    tera.register_filter("first", first_filter);
    tera.register_filter("e", escape_filter);
    tera.register_filter("escape", escape_filter);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_debug() {
        let result = format_decimal(50000.0, 0);
        eprintln!("DEBUG format_decimal(50000.0, 0) = {}", result);
        // Expected: "50,000"
    }

    #[test]
    fn test_decimal_one() {
        assert_eq!(format_decimal(1000.0, 0), "1,000");
    }

    #[test]
    fn test_decimal_zero() {
        assert_eq!(format_decimal(0.0, 0), "0");
    }
}
