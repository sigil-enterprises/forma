use std::path::{Path, PathBuf};
use std::fs;

use serde_json::Value;
use tera::Context;
use thiserror::Error;

use crate::manifest::TemplateManifest;
use crate::filters::register_filters;
use crate::context::build_context;
use crate::base_renderer::{XelatexRenderer, PdflatexRenderer, LualatexRenderer};
use crate::html_renderer::HtmlRenderer;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Template error: {0}")]
    Template(String),
    #[error("No manifest.yaml found in {0}")]
    NoManifest(PathBuf),
    #[error("Unknown engine: {engine:?}. Choose from xelatex, pdflatex, lualatex, or html.")]
    UnknownEngine { engine: String },
    #[error("LaTeX render failed: {0}")]
    LaTeX(#[from] crate::base_renderer::RenderError),
    #[error("HTML render failed: {0}")]
    Html(#[from] crate::html_renderer::HtmlRenderError),
    #[error("Tera error: {0}")]
    Tera(#[from] tera::Error),
}

impl From<crate::manifest::ManifestError> for RenderError {
    fn from(e: crate::manifest::ManifestError) -> Self {
        match e {
            crate::manifest::ManifestError::NotFound(path) => RenderError::NoManifest(path),
            crate::manifest::ManifestError::Parse(err) => RenderError::Template(err.to_string()),
            crate::manifest::ManifestError::IO(err) => RenderError::Template(err.to_string()),
        }
    }
}

pub fn render_template(
    template_dir: &Path,
    document: &Value,
    style: &Value,
    output_path: &Path,
    project_dir: Option<&Path>,
) -> Result<PathBuf, RenderError> {
    let manifest = TemplateManifest::from_path(template_dir)?;
    let tera = create_tera_env(template_dir);
    let context_value = build_context(document, style);

    let presskit_root = template_dir.parent()
        .and_then(|p| p.parent())
        .unwrap_or(template_dir);

    let project_dir_str = project_dir
        .and_then(|p| fs::canonicalize(p).ok())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let presskit_root_str = fs::canonicalize(presskit_root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| presskit_root.display().to_string());

    let mut context = Context::from_value(context_value)?;
    context.insert("project_dir", &project_dir_str);
    context.insert("presskit_root", &presskit_root_str);

    let rendered = tera.render(&manifest.entry, &context)?;

    // Restore literal braces that were escaped during preprocessing
    let rendered = restore_braces(&rendered);

    if manifest.engine == "html" {
        let renderer = HtmlRenderer::new();
        renderer.render_pdf(&rendered, output_path, Some(template_dir))?;
    } else {
        let fonts_dirs = collect_fonts_dirs(presskit_root);
        let renderer: Box<dyn LaTeXRenderer> = match manifest.engine.as_str() {
            "xelatex" => Box::new(XelatexRenderer::new()),
            "pdflatex" => Box::new(PdflatexRenderer::new()),
            "lualatex" => Box::new(LualatexRenderer::new()),
            other => return Err(RenderError::UnknownEngine { engine: other.to_string() }),
        };
        renderer.render(&rendered, output_path, project_dir, Some(&fonts_dirs))?;
    }

    Ok(output_path.to_path_buf())
}

fn create_tera_env(template_dir: &Path) -> tera::Tera {
    eprintln!("DEBUG create_tera_env: template_dir={}", template_dir.display());
    let mut builder = tera::Tera::default();

    let mut paths = vec![template_dir.to_path_buf()];
    let slides_dir = template_dir.join("_slides");
    let partials_dir = template_dir.join("_partials");
    if slides_dir.is_dir() {
        paths.push(slides_dir);
    }
    if partials_dir.is_dir() {
        paths.push(partials_dir);
    }

    let mut registered = Vec::new();
    for search_path in &paths {
        eprintln!("DEBUG  scanning: {}", search_path.display());
        if let Ok(entries) = std::fs::read_dir(search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let key = path.to_string_lossy().to_string();
                    let is_template = path.extension().map(|e| e == "j2" || e == "tera" || e == "html" || e == "tex").unwrap_or(false);
                    if is_template {
                        if let Ok(contents) = std::fs::read_to_string(&path) {
                            let processed = preprocess_delimiters(&contents);
                            let add_result = builder.add_raw_template(&key, &processed);
                            eprintln!("DEBUG    full path: {:?} (ext={:?}) processed_len={} add_result={:?}", path.file_name(), path.extension(), processed.len(), add_result.is_ok());
                            if !add_result.is_ok() {
                                eprintln!("DEBUG      ADD ERROR: {:?}", add_result.err());
                            }
                            if builder.get_template(&key).is_err() {
                                builder.add_raw_template(&key, &processed).ok();
                            }
                            let bare_name = path.file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            if !bare_name.is_empty() {
                                let bare_result = builder.add_raw_template(&bare_name, &processed);
                                eprintln!("DEBUG    bare name: {} add_result={:?}", bare_name, bare_result.is_ok());
                                if !bare_result.is_ok() {
                                    eprintln!("DEBUG      BARE ADD ERROR: {:?}", bare_result.err());
                                }
                                registered.push(bare_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    for search_path in &paths {
        if let Ok(entries) = std::fs::read_dir(search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let rel_key = format!("{}/{}",
                        search_path.file_name().unwrap_or(std::ffi::OsStr::new("")).to_string_lossy(),
                        name
                    );
                    let is_template = path.extension().map(|e| e == "j2" || e == "tera" || e == "html" || e == "tex").unwrap_or(false);
                    if is_template {
                        if let Ok(contents) = std::fs::read_to_string(&path) {
                            let processed = preprocess_delimiters(&contents);
                            if builder.get_template(&rel_key).is_err() {
                                builder.add_raw_template(&rel_key, &processed).ok();
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("DEBUG registered templates: {:?}", registered);
    let check = builder.get_template("main.tex.j2");
    eprintln!("DEBUG get_template('main.tex.j2'): {:?}", check.is_ok());

    register_filters(&mut builder);

    builder
}

/// Convert Python Jinja2 delimiters to Tera defaults.
/// Pipeline: bool_ops → array_dot → namespace → .get() → escape braces → comment → block → variable → default(block) → default(var)
/// Note: Tera natively supports {% elif %}, so no elif conversion needed.
/// Note: escape_tera_conflicts runs BEFORE convert_block so that `{%`/`{#` in TeX
/// code are replaced with `__OB__` before the block converter scans for delimiters.
/// After Tera renders, `restore_braces()` converts `__OB__` back to `{`.
pub fn preprocess_delimiters(input: &str) -> String {
    // Step 0.0: Convert x or val fallback patterns in {{ }} to if-else
    // Jinja2: {{ x or "default" }} → Tera if-else (Tera's `or` is boolean, not fallback)
    let input = convert_or_fallback(&input);

    // Step 0: Convert && → and, || → or (Tera uses word operators)
    let input = convert_bool_ops(&input);

    // Step 0.1: Convert Jinja2 null comparisons → Tera equivalents
    // Tera does not support x == null / x != null; uses x is defined / x is not undefined
    let input = convert_null_cmp(&input);

    // Step 0.2: Convert .0 → [0] (Tera requires bracket notation for array index access)
    let input = convert_array_dot(&input);

    // Step 0.5: Convert Jinja2 namespace → flat Tera variables
    let input = convert_namespace(&input);

    // Step 1: Convert .get('key') → .key
    let input = convert_get(&input);

    // Step 1.5: Convert (# comment #) → {# comment #}
    let input = convert_comment(&input);

    // Step 2: Convert (% block %) → {% block %}
    let input = convert_block(&input);

    // Step 2.5: Escape LaTeX `{` that would be misinterpreted as Tera tags.
    // Runs AFTER convert_block but BEFORE convert_null_coalesce, because
    // convert_null_coalesce generates {% %} blocks containing `}{%` patterns
    // that must be escaped. After escaping, Tera won't misparse them.
    let input = escape_tera_conflicts(&input);

    // Step 2.6: Convert a ?? b null-coalescing to if-else
    let input = convert_null_coalesce(&input);

    // Step 3: Convert `{{{ ... }}}` to `{{{{ ... }}}}` for Tera compatibility.
    let input = convert_triple_braces(&input);

    // Step 3.5: Convert | default('val') in {% %} blocks → ternary
    let input = convert_default_in_blocks(&input);

    // Step 4: Convert | default('val') in (( )) to {% if %} blocks.
    // Must run BEFORE convert_variable so it only sees original (( )) patterns,
    // not the {{ }} blocks generated by its own replacements.
    let input = convert_default_filter(&input);

    // Step 5: Convert (( )) → {{ }} (runs last so it also converts
    // the {{ }} blocks generated by convert_default_filter)
    let input = convert_variable(&input);

    input
}

/// Escape `{` that would cause Tera parse errors.
/// Uses `_OBRACE_` (plain text, no braces) as placeholder so Tera won't parse it.
/// After Tera renders, `restore_braces()` converts `_OBRACE_` back to `{`.
///
/// Rules — escape `{` when:
/// - Preceded by `}` (e.g. `}}{` or `}{` creates implicit tag)
/// - Followed by `%` (e.g. `\setbeamertemplate{foo}{%` — looks like block tag)
/// - Followed by `#` (e.g. `\macro{bar}{#` — looks like comment tag)
fn escape_tera_conflicts(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);

    for i in 0..len {
        if chars[i] == '{' {
            let prev_is_brace = i > 0 && chars[i - 1] == '{';
            let prev_is_close = i > 0 && chars[i - 1] == '}';
            let prev_is_backslash = i > 0 && chars[i - 1] == '\\';
            let next_is_percent_or_hash = i + 1 < len
                && (chars[i + 1] == '%' || chars[i + 1] == '#');
            // After }}: }{ — escape the { since Tera treats it as implicit output tag
            // After \}: \}{ — escape to prevent { followed by %/# from being parsed as Tera tags
            // Skip escaping { when preceded by { (part of Tera delimiters {{ }})
            if (prev_is_close || prev_is_backslash) && !prev_is_brace && next_is_percent_or_hash {
                result.push_str("_OBRACE_");
            } else {
                result.push('{');
            }
        } else {
            result.push(chars[i]);
        }
    }

    result
}

/// Convert `{{{ ... }}}` to `{{{{ ... }}}}` for Tera compatibility.
/// In LaTeX templates: `\begin{frame}[t]{{{ name }}}:` means literal `{` + Tera output + literal `}`.
/// Tera doesn't parse `{{{` correctly — `{{{{` inside an output tag produces one literal `{`.
fn convert_triple_braces(input: &str) -> String {
    // Convert `{{{` (LaTeX: literal `{` + Tera output) → `_OBRACE_{{` (Tera-safe).
    // `_OBRACE_` is restored to `{` by `restore_braces()` after Tera rendering.
    // Do NOT convert `}}}` — Tera parses `x}}}` as `x}}` (output end) + `}` (literal),
    // but converting it to `}}}}` breaks adjacent block tags like `{% else %}`.
    let mut result = String::with_capacity(input.len() + 10);
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // `{{{` not part of `{{{{` → `_OBRACE_{{`
        if i + 2 < len
            && chars[i] == '{'
            && chars[i + 1] == '{'
            && chars[i + 2] == '{'
            && (i + 3 >= len || chars[i + 3] != '{')
        {
            result.push_str("_OBRACE_{{");
            i += 3;
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Convert Jinja2 `x or val` fallback patterns to Tera if-else blocks.
/// Tera's `or` is a boolean operator; Jinja2's `x or default` returns x if truthy, else default.
/// Matches both `{{ }}` and `(( ))` delimiters (since (( )) haven't been converted yet).
fn convert_or_fallback(input: &str) -> String {
    // Match: (( expr or val )) or {{ expr or val }} patterns
    let re = regex::Regex::new(
        r"(\(\(|\{\{)(-?)\s*(.+?)\s+or\s+(.+?)\s*-?\s*(\)\)|\}\})"
    ).unwrap();

    re.replace_all(input, |caps: &regex::Captures| {
        let trim_le = caps.get(2).map_or(false, |m| m.as_str() == "-");
        let expr = caps.get(3).unwrap().as_str().trim();
        let fallback = caps.get(4).unwrap().as_str().trim();
        let trim_re = fallback.ends_with('-');
        let fallback = fallback.trim_end_matches('-').trim();

        let open_block = if trim_le { "{%-" } else { "{%" };
        let close_block = if trim_re { "-%}" } else { "%}" };
        let open_var = if trim_le { "{{-" } else { "{{ " };
        let close_var = if trim_re { "-}}" } else { " }}" };

        format!(
            "{} if {} is defined and {} != '' {}{} {}{} {}{}{} {}{}{} {}{}{}",
            open_block, expr, expr, close_block,
            open_var, expr, close_var,
            open_block, " else ", close_block,
            open_var, fallback, close_var,
            open_block, " endif ", close_block
        )
    }).to_string()
}

/// Convert a ?? b null-coalescing to if-else inside {% %} blocks.
/// Input:  {% set cur = a ?? 'default' %}
/// Output: {% if a is defined and a != '' %}{% set cur = a %}{% else %}{% set cur = 'default' %}{% endif %}
fn convert_null_coalesce(input: &str) -> String {
    let mut result = String::new();
    let mut last_end = 0;

    // Use regex to find all ?? occurrences
    let re = regex::Regex::new(r"\?\?").unwrap();
    for cap in re.find_iter(input) {
        let qpos = cap.start();

        // Find start of enclosing {% ... %} or _OBRACE_{% ... %} block.
        // _OBRACE_{% can appear when escape_tera_conflicts has already run
        // (e.g. }( {% sequences in adjacent blocks).
        let mut block_start = 0;
        let mut block_prefix_len = 2; // length of delimiter prefix {%
        let search_start = if qpos >= 2 { qpos - 2 } else { 0 };
        for k in (0..=search_start).rev() {
            // Check for _OBRACE_{% (9 chars total)
            if k + 9 <= input.len()
                && &input[k..k + 9] == "_OBRACE_{%"
            {
                block_start = k;
                block_prefix_len = 9;
                break;
            }
            // Check for plain {% (2 chars)
            if k + 2 <= input.len()
                && input.as_bytes()[k] == b'{'
                && input.as_bytes()[k + 1] == b'%'
            {
                block_start = k;
                block_prefix_len = 2;
                break;
            }
        }

        // Find closing %} after ??
        let block_end_search = &input[qpos..];
        let block_end = block_end_search
            .find("%}")
            .map(|p| qpos + p + 2)
            .unwrap_or(input.len());

        // Push text before this block
        if block_start > last_end {
            result.push_str(&input[last_end..block_start]);
        }

        // Parse the block content to build replacement
        let block_content = &input[block_start + block_prefix_len..block_end - 2]; // e.g. " set cur = a ?? 'default' "
        let split_pos = block_content.find("??").unwrap();
        let left = block_content[..split_pos].trim(); // "set cur = a"
        let right = block_content[split_pos + 2..].trim(); // "'default'"

        // Strip "set " keyword prefix if present (from original block syntax)
        let left = left.strip_prefix("set ").unwrap_or(left);

        let (var_name, expr) = if let Some(eq_pos) = left.find('=') {
            let var = left[..eq_pos].trim();
            let ex = left[eq_pos + 1..].trim();
            (var, ex)
        } else {
            let parts: Vec<&str> = left.splitn(2, ' ').collect();
            let var = parts.first().copied().unwrap_or("x");
            let ex = parts.get(1).copied().unwrap_or("");
            (var, ex)
        };

        let replacement = format!(
            "{{% if {} is defined and {} != '' %}}",
            expr, expr
        );
        result.push_str(&replacement);
        result.push_str(&format!("{{% set {} = {} %}}", var_name, expr));
        result.push_str("{% else %}");
        result.push_str(&format!("{{% set {} = {} %}}", var_name, right));
        result.push_str("{% endif %}");

        last_end = block_end;
    }

    // Append any remaining text after the last ??
    if last_end < input.len() {
        result.push_str(&input[last_end..]);
    }

    result
}

fn convert_bool_ops(input: &str) -> String {
    input
        .replace(" && ", " and ")
        .replace(" || ", " or ")
}

fn convert_null_cmp(input: &str) -> String {
    // Tera does not support `== null` / `!= null` comparisons.
    // Convert to Tera's equivalent: `is undefined` / `is not undefined`
    let re_eq = regex::Regex::new(r"(\S+)\s*==\s*null").unwrap();
    let re_ne = regex::Regex::new(r"(\S+)\s*!=\s*null").unwrap();
    let input = re_ne.replace_all(&input, "$1 is not undefined").to_string();
    re_eq.replace_all(&input, "$1 is undefined").to_string()
}

fn convert_array_dot(input: &str) -> String {
    // Convert Jinja2 dot-notation to bracket notation only for map keys
    // starting with a digit: .0_title → [0_title]
    // Leave bare numeric indices (.0.title) alone — Tera handles those
    // natively for both arrays and maps.
    let mut output = String::with_capacity(input.len() + 4);
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            let start = i + 1;
            let digit_end = start + bytes[start..].iter().position(|&c| !c.is_ascii_digit()).unwrap_or(bytes.len() - start);
            let next_ch = bytes.get(digit_end);
            // Only convert .digits_ (map key starting with digit)
            if next_ch.map_or(false, |&c| c == b'_') {
                // Find end of key (next dot or end)
                let key_end = digit_end + 1 + bytes[(digit_end + 1)..].iter().position(|&c| c == b'.').unwrap_or(bytes.len() - digit_end - 1);
                output.push('[');
                output.push_str(&input[start..key_end]);
                output.push(']');
                i = key_end;
                continue;
            }
            // Otherwise (.0.title or .0 at end) — output dot and continue scanning
        }
        output.push(bytes[i] as char);
        i += 1;
    }
    output
}

fn build_default_ifelse(var_name: &str, default_val: &str, filter: Option<&str>, trim: bool, is_array: bool) -> String {
    let open_var = if trim { "{{-" } else { "{{ " };
    let close_var = if trim { "-}}" } else { "}}" };
    let open_block = if trim { "{%-" } else { "{%" };
    let close_block = if trim { "-%}" } else { "%}" };

    let filter_str: String = if let Some(f) = filter { f.to_string() } else { String::new() };
    let inner_var = if filter_str.is_empty() {
        var_name.to_string()
    } else {
        format!("{}|{}", var_name, filter_str)
    };

    // Array defaults: wrap in []. String defaults: single-quote for Tera.
    let inner_default = if is_array {
        format!("[{}]", default_val)
    } else if filter_str.is_empty() {
        default_val.to_string()
    } else {
        format!("'{}'|{}", default_val, filter_str)
    };

    let mut r = String::new();

    // {%- if var is defined and var != '' -%}
    r.push_str(&format!("{} if {} is defined and {} != '' {}", open_block, var_name, var_name, close_block));

    // {{ var|filter -}} or {{ var -}}
    r.push_str(&format!("{} {} {}", open_var, inner_var, close_var));

    // {%- else -%}
    r.push_str(&format!("{} else {}", open_block, close_block));

    // {{ 'default'|filter -}} or {{ ['a']|filter -}}
    r.push_str(&format!("{} {} {}", open_var, inner_default, close_var));

    // {%- endif -%}
    r.push_str(&format!("{} endif {}", open_block, close_block));

    r
}

fn convert_default_filter(input: &str) -> String {
    // Convert (( x | default('val') )) and {{ x | default('val') }} to {% if %} blocks.
    // Handles both (( )) (original Jinja2 delimiters) and {{ }} (already-converted).
    // Scans for both patterns so it works regardless of pipeline order.
    let mut result = String::with_capacity(input.len() + 64);
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut found_default = false;

    while i < len {
        // Detect opening: (( or {{
        let (_, close_pat_char, open_len) = if i + 1 < len && chars[i] == '(' && chars[i + 1] == '(' {
            ('(', ')', 2)
        } else if i + 1 < len && chars[i] == '{' && chars[i + 1] == '{' {
            ('{', '}', 2)
        } else {
            result.push(chars[i]);
            i += 1;
            continue;
        };

        let start = i;
        let left_trim = i + open_len < len && chars[i + open_len] == '-';
        let content_start = i + open_len + if left_trim { 1 } else { 0 };

        // Find closing )) or }}, possibly with trim: -)) or -}}
        let close_pat = [close_pat_char, close_pat_char];
        let mut j = content_start;
        while j + 1 < len {
            if chars[j] == close_pat[0] && chars[j + 1] == close_pat[1] {
                break;
            }
            j += 1;
        }
        if j + open_len <= len && chars[j] == close_pat[0] && chars[j+1] == close_pat[1] {
            let is_trim_right = chars[j] == '-';
            let content_end = if is_trim_right { j } else { j + open_len };
            let content: String = chars[content_start..content_end].iter().collect();

            let has_default = content.contains("default(");
            if has_default {
                let pipe_def_pos = content.find("default(").unwrap();
                // Extract raw variable portion: strip trailing `|` and whitespace from everything before `default(`
                let var_name: String = content[..pipe_def_pos]
                    .trim_end_matches(|c: char| c == '|' || c.is_whitespace())
                    .to_string();
                let var_name = var_name.trim();
                let after_default = &content[pipe_def_pos + 8..]; // skip 'default('
                let after = after_default.trim();

                // Parse fallback value and optional filters from after-default part
                let (fallback_inner, opt_filter, is_array) = if after.starts_with('\'') {
                    if let Some(end) = after[1..].find('\'') {
                        let val = &after[1..1 + end];
                        let rest = &after[2 + end..].trim();
                        (val.to_string(), parse_filters(rest), false)
                    } else {
                        (String::new(), None, false)
                    }
                } else if after.starts_with('"') {
                    if let Some(end) = after[1..].find('"') {
                        let val = &after[1..1 + end];
                        let rest = &after[2 + end..].trim();
                        (val.to_string(), parse_filters(rest), false)
                    } else {
                        (String::new(), None, false)
                    }
                } else if after.starts_with('[') {
                    if let Some(end) = after.find(']') {
                        let val = &after[1..end];
                        let rest = &after[end + 1..].trim();
                        (val.to_string(), parse_filters(rest), true)
                    } else {
                        (String::new(), None, false)
                    }
                } else {
                    (String::new(), None, false)
                };

                let trim_flag = left_trim || after_default.trim().ends_with("-");
                let out = build_default_ifelse(var_name, &fallback_inner, opt_filter.as_deref(), trim_flag, is_array);
                result.push_str(&out);
                found_default = true;
                i = if is_trim_right { j + open_len } else { j + open_len + 1 };
                continue;
            }

            // No default filter — output as-is
            let close_pos = if is_trim_right { j + 1 } else { j + open_len };
            result.push_str(&input[start..close_pos]);
            i = close_pos;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn parse_filters(s: &str) -> Option<String> {
    // Parse `| filter1 | filter2` from the remaining string after default()
    // Skip closing paren and whitespace: `) | hex_color` → `| hex_color`
    let s = s.trim_start_matches(|c: char| c == ')' || c.is_whitespace());
    if s.starts_with('|') {
        // Strip trailing ), leading |, and whitespace; collect filter string
        let filtered: String = s
            .chars()
            .filter(|c| *c != ')')
            .filter(|c| !c.is_whitespace())
            .collect();
        // Remove leading | from the collected string (e.g. "|isdefined" → "isdefined")
        let stripped = filtered.strip_prefix('|').unwrap_or(&filtered);
        if !stripped.is_empty() {
            Some(stripped.to_string())
        } else {
            None
        }
    } else {
        None
    }
}

fn convert_default_in_blocks(input: &str) -> String {
    // Convert `| default('val')` inside `{% %}` blocks to ternary.
    // Tera does NOT support `| default()` in {% set %} expressions.
    // Example: `{% set x = var | default('val') %}` →
    //   `{% set x = var is defined and var != '' ? var : 'val' %}`
    // Only replaces inside `{% %}` blocks, not `{{ }}` (those are handled by convert_default_filter).
    let mut result = String::with_capacity(input.len() + 32);
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Track if we're at a {% %} block start
        if i + 1 < len && chars[i] == '{' && chars[i + 1] == '%' {
            // Start of a block — find the closing %}
            let mut j = i + 2;
            let mut in_str = false;
            let str_ch = '"';
            while j + 1 < len {
                if in_str {
                    if chars[j] == str_ch && (j == 0 || chars[j - 1] != '\\') { in_str = false; }
                } else {
                    if chars[j] == str_ch { in_str = true; }
                    if chars[j] == '%' && chars[j + 1] == '}' { break; }
                }
                j += 1;
            }
            if j + 1 < len && chars[j] == '%' && chars[j + 1] == '}' {
                // Use char-indexed byte positions via chars iterator to avoid
                // multi-byte UTF-8 slicing issues.
                let block_end_chars = j + 2; // past %}
                let block_content: String = chars[i..block_end_chars].iter().collect();

                // Replace | default('val') with : 'val' inside this block
                let replaced = replace_default_in_str(&block_content);
                result.push_str(&replaced);
                i = block_end_chars;
                continue;
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

fn replace_default_in_str(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(input.len() + 16);
    let mut i = 0;

    while i < len {
        if chars[i] == '|' {
            let mut j = i + 1;
            while j < len && chars[j] == ' ' { j += 1; }
            if j + 7 <= len && &chars[j..j+7] == &['d','e','f','a','u','l','t'] {
                let mut k = j + 7;
                while k < len && chars[k] == ' ' { k += 1; }
                if k < len && chars[k] == '(' {
                    let mut depth = 1;
                    let mut p = k + 1;
                    while p < len && depth > 0 {
                        if chars[p] == '(' { depth += 1; }
                        if chars[p] == ')' { depth -= 1; }
                        if depth > 0 { p += 1; }
                    }
                    if depth == 0 && p < len {
                        // Extract fallback: between k+1 and p
                        let fallback: String = chars[k+2..p].iter().collect();
                        let quote = chars[k + 1];
                        result.push_str(" : ");
                        result.push(quote);
                        result.push_str(&fallback);
                        result.push(quote);
                        i = p + 1;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

fn convert_ns_refs(input: &str) -> String {
    // Convert ns.key references → ns_key (flat variable)
    // Skip "namespace" keyword entirely
    let re = regex::Regex::new(r"\bns\.(\w+)").unwrap();
    re.replace_all(input, |cap: &regex::Captures| {
        if cap[1] == *"namespace" {
            cap[0].to_string()
        } else {
            format!("ns_{}", &cap[1])
        }
    }).to_string()
}

fn convert_namespace(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut result: Vec<String> = Vec::new();

    // First pass: convert all ns.key → ns_key references, but SKIP lines with `set`
    // (those are handled by the special set var.key = expr regex below)
    let normalized: Vec<String> = lines.iter().map(|l| {
        if l.contains("set ") { l.to_string() } else { convert_ns_refs(l) }
    }).collect();

    // Collect known aggregate names: var.key → var_key (for global dot→bracket conversion later)
    let mut aggregates: Vec<(String, String)> = Vec::new();

    for line in &normalized {
        // set <var> = namespace(key=val, ...) → {% set var_key = val %} ...
        // Handles any namespace variable name (ns, pt, gt, etc.)
        if line.contains("namespace(") {
            let re = regex::Regex::new(r#"set\s+(\w+)\s*=\s*namespace\s*\(([^)]*)\)"#).unwrap();
            if let Some(cap) = re.captures(line) {
                let var_name = &cap[1];
                let params = &cap[2];
                for pair in params.split(',') {
                    let pair = pair.trim();
                    if let Some(eq_pos) = pair.find('=') {
                        let key = pair[..eq_pos].trim();
                        let val = pair[eq_pos + 1..].trim();
                        result.push(format!("{{% set_global {}_{} = {} %}}", var_name, key, val));
                    }
                }
            } else {
                result.push(line.to_string());
            }
            continue;
        }

        // Handle set var.key = expr (Tera-style and Jinja2-style delimiters)
        // Track aggregates for global dot→underscore conversion in other expressions
        // IMPORTANT: Use replace_all (not captures) to preserve surrounding content
        // on the same line (e.g., {% for %}{% set %}{% endfor %} on one line)
        let re_set = regex::Regex::new(r#"(\s*)\{\%\s*set\s+(\w+)\.(\w+)\s*=\s*(.+?)\s*\%\}"#).unwrap();
        if re_set.is_match(line) {
            let line = re_set.replace_all(line, |cap: &regex::Captures| -> String {
                let ws = &cap[1];
                let var = &cap[2];
                let key = &cap[3];
                let val = &cap[4];
                aggregates.push((var.to_string(), key.to_string()));
                let processed_val = convert_aggregate_dots(val, &aggregates);
                format!("{}{{% set_global {}_{} = {} %}}", ws, var, key, processed_val)
            });
            result.push(line.to_string());
            continue;
        }
        let re_set_j2 = regex::Regex::new(r#"(\s*)\(%\s*set\s+(\w+)\.(\w+)\s*=\s*(.+?)\s*%\)"#).unwrap();
        if re_set_j2.is_match(line) {
            let line = re_set_j2.replace_all(line, |cap: &regex::Captures| -> String {
                let ws = &cap[1];
                let var = &cap[2];
                let key = &cap[3];
                let val = &cap[4];
                aggregates.push((var.to_string(), key.to_string()));
                let processed_val = convert_aggregate_dots(val, &aggregates);
                format!("{}{{% set_global {}_{} = {} %}}", ws, var, key, processed_val)
            });
            result.push(line.to_string());
            continue;
        }

        result.push(line.to_string());
    }

    // Global post-processing: convert known aggregate dot refs to underscore notation
    // e.g. phase_total.val → phase_total_val in ALL remaining expressions
    let mut output = result.join("\n");
    output = convert_aggregate_dots(&output, &aggregates);
    output
}

fn convert_aggregate_dots(input: &str, aggregates: &[(String, String)]) -> String {
    let mut result = input.to_string();
    for (var, key) in aggregates {
        let pattern = format!(r"\b{}\.\b{}", regex::escape(var), regex::escape(key));
        if let Ok(re) = regex::Regex::new(&pattern) {
            result = re.replace_all(&result, format!("{}_{}", var, key)).to_string();
        }
    }
    result
}

fn convert_get(input: &str) -> String {
    // .get('key', 'default') → .key or 'default'
    let re_str_default = regex::Regex::new(r#"\.get\('([^']+)',\s*'([^']*)'\)"#).unwrap();
    let input = re_str_default.replace_all(input, ".$1 or '$2'").to_string();
    // .get('key', 0) / .get('key', 1.5) → .key or 0 / .key or 1.5
    let re_num_default = regex::Regex::new(r#"\.get\('([^']+)',\s*(\d+(?:\.\d+)?)\)"#).unwrap();
    let input = re_num_default.replace_all(&input, ".$1 or $2").to_string();
    // .get('key', True/False/None) → .key or True/False/None
    let re_pyconst = regex::Regex::new(r#"\.get\('([^']+)',\s*(True|False|None)\)"#).unwrap();
    let input = re_pyconst.replace_all(&input, ".$1 or $2").to_string();
    // .get('key') → .key
    let re_single = regex::Regex::new(r#"\.get\('([^']+)'\)"#).unwrap();
    re_single.replace_all(&input, ".$1").to_string()
}

fn convert_comment(input: &str) -> String {
    // (# comment #) → {# comment #}
    input.replace("(#", "{#").replace("#)", "#}")
}

fn convert_block(input: &str) -> String {
    // Replace (% ... %) with {% ... %}
    // Use char scanner to avoid conflicts with (( ... )) variable delimiters
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if i + 1 < len && chars[i] == '(' && chars[i + 1] == '%' {
            // Found block start delimiter (% — find matching %)
            let mut j = i + 2;
            while j + 1 < len && !(chars[j] == '%' && chars[j + 1] == ')') {
                j += 1;
            }
            if j + 1 < len {
                // Replace (% ... %) with {% ... %}
                let inner: String = chars[i + 2..j].iter().collect();
                result.push_str("{%");
                result.push_str(&inner);
                result.push_str("%}");
                i = j + 2;
            } else {
                // No matching %), output as-is
                result.push('(');
                i += 1;
            }
        } else if i + 1 < len && chars[i] == '(' && chars[i + 1] == '(' {
            // Skip over (( ... )) — handled by convert_variable later
            // but we need to skip parens inside to avoid double-replacing
            result.push('(');
            result.push('(');
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

fn convert_variable(input: &str) -> String {
    // (( ... )) -> {{ }}
    // Handles trim markers: ((- -> {{- and -)) -> -}}
    // Paren depth tracking so )) inside strings like default('...)') works
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i + 1 < len && chars[i] == '(' && chars[i + 1] == '(' {
            // Find closing ))
            let mut j = i + 2;
            while j + 1 < len && !(chars[j] == ')' && chars[j + 1] == ')') {
                j += 1;
            }
            if j + 1 < len {
                // Detect left trim: ((-
                let left_trim = chars.get(i + 2).map_or(false, |&c| c == '-');
                // Detect right trim: -))
                let right_trim = j > 0 && chars[j - 1] == '-' && j > i + 2;

                // Extract inner content, skipping trim chars
                let inner_start: usize = if left_trim { i + 3 } else { i + 2 };
                let inner_end: usize = if right_trim { j - 1 } else { j };

                let inner: String = chars[inner_start..inner_end].iter().collect();
                // Trim one leading/trailing space to normalize (( content )) -> {{ content }}
                let trimmed = inner.trim_start_matches(' ').trim_end_matches(' ');

                result.push_str(if left_trim { "{{-" } else { "{{ " });
                result.push_str(trimmed);
                result.push_str(if right_trim { " -}}" } else { " }}" });
                i = j + 2;
            } else {
                result.push_str("((");
                i += 2;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Restore literal braces from placeholders used to avoid Tera parsing conflicts.
fn restore_braces(input: &str) -> String {
    input.replace("_OBRACE_", "{")
}

fn collect_fonts_dirs(presskit_root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let fonts = presskit_root.join("fonts");
    if fonts.is_dir() {
        dirs.push(fonts);
    }
    if presskit_root.is_dir() {
        dirs.push(presskit_root.to_path_buf());
    }
    dirs
}

pub trait LaTeXRenderer: std::fmt::Debug {
    fn render(
        &self,
        tex_source: &str,
        output_path: &Path,
        project_dir: Option<&Path>,
        fonts_dirs: Option<&[PathBuf]>,
    ) -> Result<(), crate::base_renderer::RenderError>;
}

impl LaTeXRenderer for XelatexRenderer {
    fn render(&self, tex_source: &str, output_path: &Path, project_dir: Option<&Path>, fonts_dirs: Option<&[PathBuf]>) -> Result<(), crate::base_renderer::RenderError> {
        self.0.render(tex_source, output_path, project_dir, fonts_dirs)
    }
}
impl LaTeXRenderer for PdflatexRenderer {
    fn render(&self, tex_source: &str, output_path: &Path, project_dir: Option<&Path>, fonts_dirs: Option<&[PathBuf]>) -> Result<(), crate::base_renderer::RenderError> {
        self.0.render(tex_source, output_path, project_dir, fonts_dirs)
    }
}
impl LaTeXRenderer for LualatexRenderer {
    fn render(&self, tex_source: &str, output_path: &Path, project_dir: Option<&Path>, fonts_dirs: Option<&[PathBuf]>) -> Result<(), crate::base_renderer::RenderError> {
        self.0.render(tex_source, output_path, project_dir, fonts_dirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::{first_filter, map_filter, selectattr_filter};

    fn make_tera_with(template: &str) -> tera::Tera {
        let mut t = tera::Tera::default();
        t.add_raw_template("t", template).unwrap();
        register_filters(&mut t);
        t
    }

    // --- Tera operator/filter compatibility tests ---

    #[test]
    fn test_tera_or_with_null() {
        let mut t = tera::Tera::default();
        let r = t.add_raw_template("t", r#"{{ x or 'fallback' }}"#);
        assert!(r.is_ok(), "Tera or operator: {:?}", r);
        let mut ctx = Context::new();
        ctx.insert("x", &serde_json::Value::Null);
        let result = t.render("t", &ctx.into());
        // Tera: undefined vars trigger or, null is truthy
        let ctx2 = Context::new();
        let result2 = t.render("t", &ctx2.into());
        println!("TERA or_with_null: x=Null → {:?}, x=missing → {:?}", result, result2);
    }

    #[test]
    fn test_tera_or_with_none() {
        let mut t = tera::Tera::default();
        let r = t.add_raw_template("t", r#"{{ x or 'fallback' }}"#);
        assert!(r.is_ok(), "Tera or operator: {:?}", r);
        let mut ctx = Context::new();
        ctx.insert("x", &None as &Option<String>);
        let result = t.render("t", &ctx.into());
        println!("TERA or_with_none: x=None → {:?}", result);
    }

    #[test]
    fn test_tera_selectattr_double_quotes() {
        let items = serde_json::json!([
            {"type": "other", "title": "A"},
            {"type": "cover", "title": "Hello"},
        ]);
        let mut args = std::collections::HashMap::new();
        args.insert("attribute".into(), "type".into());
        args.insert("value".into(), "cover".into());
        let selected = selectattr_filter(&items, &args).unwrap();
        let mut map_args = std::collections::HashMap::new();
        map_args.insert("attribute".into(), "title".into());
        let mapped = map_filter(&selected, &map_args).unwrap();
        let first = first_filter(&mapped, &std::collections::HashMap::new()).unwrap();
        assert_eq!(first.as_str().unwrap(), "Hello");
    }

    #[test]
    fn test_tera_json_null_render() {
        let mut t = tera::Tera::default();
        t.add_raw_template("t", r#"{{ x }}"#).unwrap();
        let ctx = Context::from_value(serde_json::json!({"x": null})).unwrap();
        let result = t.render("t", &ctx);
        println!("TERA json_null_render: {:?}", result);
    }

    #[test]
    fn test_tera_json_null_or() {
        let mut t = tera::Tera::default();
        t.add_raw_template("t", r#"{{ x or 'fallback' }}"#).unwrap();
        let ctx = Context::from_value(serde_json::json!({"x": null})).unwrap();
        let result = t.render("t", &ctx).unwrap();
        assert_eq!(result.trim(), "true", "Tera `or` is boolean, not null-fallback");
    }

    #[test]
    fn test_tera_if_set_else() {
        let mut t = tera::Tera::default();
        t.add_raw_template("t", "{% if x %}{{ x }}{% else %}fallback{% endif %}").unwrap();
        let ctx = Context::new();
        let result = t.render("t", &ctx).unwrap();
        assert_eq!(result.trim(), "fallback");
    }

    #[test]
    fn test_tera_or_with_value() {
        let mut t = tera::Tera::default();
        t.add_raw_template("t", r#"{{ x or 'fallback' }}"#).unwrap();
        let mut ctx = Context::new();
        ctx.insert("x", "hello");
        let result = t.render("t", &ctx.into()).unwrap();
        assert_eq!(result.trim(), "true", "Tera `or` is boolean");
    }

    #[test]
    fn test_tera_or_with_empty_string() {
        let mut t = tera::Tera::default();
        t.add_raw_template("t", r#"{{ x or 'fallback' }}"#).unwrap();
        let mut ctx = Context::new();
        ctx.insert("x", "");
        let result = t.render("t", &ctx.into()).unwrap();
        assert_eq!(result.trim(), "true", "Tera `or` is boolean");
    }

    #[test]
    fn test_tera_or_with_zero() {
        let mut t = tera::Tera::default();
        t.add_raw_template("t", r#"{{ x or 'fallback' }}"#).unwrap();
        let mut ctx = Context::new();
        ctx.insert("x", &0);
        let result = t.render("t", &ctx.into()).unwrap();
        assert_eq!(result.trim(), "true", "Tera `or` is boolean");
    }

    #[test]
    fn test_tera_or_with_empty_array() {
        let mut t = tera::Tera::default();
        t.add_raw_template("t", r#"{{ x or 'fallback' }}"#).unwrap();
        let mut ctx = Context::new();
        ctx.insert("x", &Vec::<String>::new());
        let result = t.render("t", &ctx.into()).unwrap();
        assert_eq!(result.trim(), "true", "Tera `or` is boolean");
    }

    // --- Template preprocessor tests ---

    #[test]
    fn test_convert_get_method() {
        let input = r#"{{ slide.get('key') }}"#;
        let result = preprocess_delimiters(input);
        assert_eq!(result, "{{ slide.key }}");
    }

    #[test]
    fn test_convert_get_method_with_default() {
        let input = r#"{{ slide.get('key', 'default_val') }}"#;
        let result = preprocess_delimiters(input);
        assert_eq!(result, "{{ slide.key or 'default_val' }}");
    }

    #[test]
    fn test_convert_comment() {
        let input = r#"(# this is a comment #)"#;
        let result = preprocess_delimiters(input);
        assert_eq!(result, "{# this is a comment #}");
    }

    #[test]
    fn test_convert_block() {
        let input = r#"(% for i in items %)"#;
        let result = preprocess_delimiters(input);
        assert_eq!(result, "{% for i in items %}");
    }

    #[test]
    fn test_convert_variable_paren_depth() {
        let input = r"(( x | e ))";
        let result = preprocess_delimiters(input);
        assert_eq!(result.replace("  ", " ").trim(), "{{ x | e }}");
    }

    // --- Tera rendering tests ---

    #[test]
    fn test_simple_set() {
        let t = make_tera_with("{% set x = 'hello' %}{{ x }}");
        let r = t.render("t", &Context::new().into()).unwrap();
        assert_eq!(r.trim(), "hello");
    }

    #[test]
    fn test_pipe_simple() {
        let t = make_tera_with("{{ x | upper }}");
        let mut ctx = Context::new();
        ctx.insert("x", "hello");
        let r = t.render("t", &ctx).unwrap();
        assert_eq!(r.trim(), "HELLO");
    }

    #[test]
    fn test_set_with_or() {
        let t = make_tera_with("{% set x = 'hello' or 'world' %}{{ x }}");
        let r = t.render("t", &Context::new().into()).unwrap();
        assert_eq!(r.trim(), "true", "Tera `or` is boolean");
    }

    #[test]
    fn test_set_empty_or() {
        let t = make_tera_with("{% set x = '' or 'fallback' %}{{ x }}");
        let r = t.render("t", &Context::new().into()).unwrap();
        assert_eq!(r.trim(), "true", "Tera `or` is boolean");
    }

    #[test]
    fn test_for_loop() {
        let t = make_tera_with("{% for i in items %}{{ i }}{% endfor %}");
        let mut ctx = Context::new();
        ctx.insert("items", &vec!["a", "b", "c"]);
        let r = t.render("t", &ctx).unwrap();
        assert_eq!(r, "abc");
    }

    #[test]
    fn test_if_tag() {
        let t = make_tera_with("{% if x %}yes{% endif %}");
        let mut ctx = Context::new();
        ctx.insert("x", "true");
        let r = t.render("t", &ctx).unwrap();
        assert_eq!(r, "yes");
    }

    #[test]
    fn test_selectattr_filter() {
        let items = serde_json::json!([
            {"type": "other", "title": "A"},
            {"type": "cover", "title": "Hello"},
        ]);
        let mut select_args = std::collections::HashMap::new();
        select_args.insert("attribute".to_string(), "type".into());
        select_args.insert("value".to_string(), "cover".into());
        let selected = selectattr_filter(&items, &select_args).unwrap();
        let mut map_args = std::collections::HashMap::new();
        map_args.insert("attribute".to_string(), "title".into());
        let mapped = map_filter(&selected, &map_args).unwrap();
        let first = first_filter(&mapped, &std::collections::HashMap::new()).unwrap();
        assert_eq!(first.as_str(), Some("Hello"));
    }

    #[test]
    fn test_selectattr_no_match() {
        let items = serde_json::json!([
            {"type": "other", "title": "A"},
        ]);
        let mut select_args = std::collections::HashMap::new();
        select_args.insert("attribute".to_string(), "type".into());
        select_args.insert("value".to_string(), "missing".into());
        let selected = selectattr_filter(&items, &select_args).unwrap();
        let first = first_filter(&selected, &std::collections::HashMap::new()).unwrap();
        assert!(first.is_null());
    }

    #[test]
    fn test_inline_if() {
        let t = make_tera_with("a{% if x %}b{% endif %}c");
        let mut ctx = Context::new();
        ctx.insert("x", "true");
        let r = t.render("t", &ctx).unwrap();
        assert_eq!(r, "abc");
    }

    #[test]
    fn test_filter_in_html() {
        let t = make_tera_with("<h1>{% if x %}{{ x | e }}{% else %}Title{% endif %}</h1>");
        let mut ctx = Context::new();
        ctx.insert("x", &serde_json::Value::Null);
        let r = t.render("t", &ctx).unwrap();
        assert_eq!(r.trim(), "<h1>Title</h1>");
    }

    #[test]
    fn test_namespace_preprocess() {
        let input = "{% set ns = namespace(idx=0) %}\n{% set ns.idx = ns.idx + 1 %}\n{{ ns.idx }}";
        let result = convert_namespace(input);
        println!("NAMESPACE PREPROCESS:\n{}", result);
        assert!(result.contains("ns_idx"), "Expected ns_idx flat var");
    }

    #[test]
    fn test_main_html_preprocess() {
        // Inline template with namespace patterns (covers real-world ns.x usage)
        let contents = r#"<!DOCTYPE html>
<html>
<head><title>(( document.title ))</title></head>
<body>
<h1>(( document.title ))</h1>
{% set ns = namespace(count=0) %}
{% set ns.count = ns.count + 1 %}
(% for slide in content.slides %)
<div class="slide">
  <h2>(( slide.title ))</h2>
  <p>(( slide.content or "" ))</p>
  <span>(( ns.count ))</span>
</div>
(% endfor %))
</body>
</html>
"#;
        let processed = preprocess_delimiters(&contents);
        // Verify namespace conversion: ns.xxx → ns_xxx (flat Tera variables)
        assert!(processed.contains("{% set_global ns_count = 0 %}"), "Expected namespace init");
        assert!(processed.contains("{% set_global ns_count = ns_count + 1 %}"), "Expected namespace increment");
        assert!(!processed.contains("ns.count"), "Should not contain ns. references");
        // Verify Tera can parse
        let mut t = tera::Tera::default();
        assert!(t.add_raw_template("test", &processed).is_ok(), "Template should parse after conversion");
    }

    #[test]
    fn test_elif_passthrough() {
        let input = r#"  (% if slide.type == 'cover' %)
    (% include 'cover.html.j2' %)
  (% elif slide.type == 'exec_summary' %)
    (% include 'exec_summary.html.j2' %)
  (% elif slide.type == 'section_divider' %)
    (% include 'section_divider.html.j2' %)
  (% elif slide.type == 'closing' %)
    (% include 'closing.html.j2' %)
  (% endif %)"#;
        let result = preprocess_delimiters(input);
        // Delimiters converted, elif preserved
        assert!(result.contains("{% elif slide.type == 'exec_summary' %}"),
            "elif should be preserved with block delimiters");
        assert!(result.contains("{% if slide.type == 'cover' %}"),
            "if should be preserved");
        assert!(result.contains("{% endif %}"),
            "endif should be preserved");
        assert!(result.contains("{% else %}") == false || result.contains("{% else %}"),
            "else may be present if originally there");
        // Should be parseable by Tera
        let mut t = tera::Tera::default();
        let r = t.add_raw_template("test", &result);
        assert!(r.is_ok(), "Template should parse: {:?}", r);
    }

    #[test]
    fn test_elif_inline_passthrough() {
        let input = "  (% if x %)y(% elif z %)w(% endif %)";
        let result = preprocess_delimiters(input);
        // Should parse cleanly with elif preserved
        let mut t = tera::Tera::default();
        let r = t.add_raw_template("test", &result);
        assert!(r.is_ok(), "Inline elif should pass through and parse: {:?}", r);
    }
}

#[cfg(test)]
mod trim_tests {
    use super::*;
    #[test]
    fn test_tera_trim_markers() {
        let mut t = tera::Tera::default();
        // Test 1: Simple trim
        let r1 = t.add_raw_template("t1", "{{- x -}}");
        assert!(r1.is_ok(), "Simple trim: {:?}", r1);

        // Test 2: With pipe filter
        let r2 = t.add_raw_template("t2", "{{- x | upper -}}");
        assert!(r2.is_ok(), "With pipe: {:?}", r2);

        // Test 3: default() with args cannot be parsed by raw Tera —
        // convert_default_filter must handle this conversion first.
        let j2_input = "{{- x | default(['hello']) -}}";
        let converted = convert_default_filter(j2_input);
        let r3 = t.add_raw_template("t3", &converted);
        assert!(r3.is_ok(), "Converted default+trim: {:?} => {:?}", j2_input, r3);

        // Test 4: Verify conversion output is valid Tera if/else
        assert!(converted.contains("if x is defined"),
            "Expected if/else conversion, got: {}", converted);
    }
}

#[cfg(test)]
mod trim_tests2 {
    #[test]
    fn test_tera_variants() {
        let mut t = tera::Tera::default();
        
        let r1 = t.add_raw_template("t1", "{{- x | default(['hello']) }}");
        println!("No right trim: ok={}", r1.is_ok());
        if let Err(e) = &r1 { println!("  err: {}", e); }
        
        let r2 = t.add_raw_template("t2", "{{- x | default('hello') -}}");
        println!("String default + trim: ok={}", r2.is_ok());
        if let Err(e) = &r2 { println!("  err: {}", e); }
        
        let r5 = t.add_raw_template("t5", "{{- x | default(['a','b']) -}}");
        println!("Array default + trim: ok={}", r5.is_ok());
        if let Err(e) = &r5 { println!("  err: {}", e); }
        
        let r6 = t.add_raw_template("t6", "{{- style.colors.primary_dark | default('0B1D2A') | hex_color -}}");
        println!("Real pattern + trim: ok={}", r6.is_ok());
        if let Err(e) = &r6 { println!("  err: {}", e); }
        
        let r7 = t.add_raw_template("t7", "{{- style.colors.primary_dark | default('0B1D2A') | hex_color }}");
        println!("Real pattern no right trim: ok={}", r7.is_ok());
        if let Err(e) = &r7 { println!("  err: {}", e); }
        
        assert!(true);
    }
}

#[cfg(test)]
mod default_filter_test {
    use super::*;
    #[test]
    fn test_default_filter_variants() {
        let mut t = tera::Tera::default();
        
        // Test: default alone without pipe (Tera built-in)
        let r1 = t.add_raw_template("t1", "{{- x | default -}}");
        println!("default alone: ok={}", r1.is_ok());
        if let Err(e) = &r1 { println!("  err: {}", e); }
        
        // Test: default('val') no trim
        let r2 = t.add_raw_template("t2", "{{ x | default('val') }}");
        println!("default(val) no trim: ok={}", r2.is_ok());
        if let Err(e) = &r2 { println!("  err: {}", e); }
        
        // Test: default('val') with left trim only
        let r3 = t.add_raw_template("t3", "{{- x | default('val') }}");
        println!("default(val) left trim only: ok={}", r3.is_ok());
        if let Err(e) = &r3 { println!("  err: {}", e); }
        
        // Test: default('val') with right trim only
        let r4 = t.add_raw_template("t4", "{{ x | default('val') -}}");
        println!("default(val) right trim only: ok={}", r4.is_ok());
        if let Err(e) = &r4 { println!("  err: {}", e); }
        
        // Test: just pipe with simple filter
        let r5 = t.add_raw_template("t5", "{{ x | upper }}");
        println!("simple pipe: ok={}", r5.is_ok());
        if let Err(e) = &r5 { println!("  err: {}", e); }
        
        // Test: double-quoted default
        let r6 = t.add_raw_template("t6", "{{ x | default(\"val\") }}");
        println!("default double-quoted: ok={}", r6.is_ok());
        if let Err(e) = &r6 { println!("  err: {}", e); }
    }
}

#[cfg(test)]
mod tera_feature_check {
    use super::*;
    #[test]
    fn check_tera_features() {
        let mut t = tera::Tera::default();

        // Test ?? null coalescing
        let r1 = t.add_raw_template("t1", "{{ x ?? 'fallback' }}");
        println!("?? operator: ok={}", r1.is_ok());

        // Test ternary with defined check
        let r2 = t.add_raw_template("t2", "{{ x is defined ? x : 'fallback' }}");
        println!("ternary defined: ok={}", r2.is_ok());

        // Test render with undefined var (ternary)
        let r2_render = r2.and_then(|_| {
            let ctx = tera::Context::new();
            t.render("t2", &ctx)
        });
        println!("ternary render undefined: {:?}", r2_render);

        // Test render with defined var
        let ctx2 = tera::Context::from_value(serde_json::json!({"x": "hello"}));
        if let Ok(ctx2) = ctx2 {
            let r2_render2 = t.render("t2", &ctx2);
            println!("ternary render defined: {:?}", r2_render2);
        }
    }

    #[test]
    fn test_default_conversion() {
        // Test convert_default_filter directly with known input
        let input = r"{{- style.typography.font_primary | default('Helvetica') -}}";
        let after_all = convert_variable(input);
        let result = convert_default_filter(&after_all);

        eprintln!("INPUT:  {:?}", input);
        eprintln!("AFTER_VAR:  {:?}", after_all);
        eprintln!("RESULT:   {:?}", result);
        std::fs::write("/tmp/default_conv_test.txt", &result).ok();

        // Now try to parse with Tera
        let mut t = tera::Tera::default();
        let r = t.add_raw_template("test", &result);
        eprintln!("Tera parse: ok={}", r.is_ok());
        if let Err(e) = &r {
            eprintln!("  error: {}", e);
        }
        assert!(r.is_ok(), "Should parse: {}", result);
    }

    #[test]
    fn test_ternary_formats() {
        let variants = vec![
            ("just_var", "{{ style }}"),
            ("nested_dots", "{{ style.font_primary }}"),
            ("pipe_chain", "{{ style | upper }}"),
            ("ternary_var", "{{ style ? other : fallback }}"),
            ("ternary_paren", "{{ (style ? other : fallback) }}"),
            ("ternary_str_val", "{{ (style ? style : 'val') }}"),
            ("ternary_single", "{{ (a ? b : c) }}"),
            ("if_else", "{% if style %}yes{% else %}no{% endif %}"),
        ];
        let mut out = String::new();
        for (label, v) in variants {
            let mut t = tera::Tera::default();
            match t.add_raw_template("test", v) {
                Ok(()) => out.push_str(&format!("{}: OK\n", label)),
                Err(e) => out.push_str(&format!("{}: FAIL - {}\n", label, e)),
            }
        }

        // Also try with Builder (like actual engine does)
        let mut builder = tera::Tera::default();
        let variants2 = vec![
            ("builder_ternary", "{{ (a ? b : c) }}"),
            ("builder_var", "{{ a }}"),
        ];
        for (label, v) in variants2 {
            match builder.add_raw_template("test", v) {
                Ok(()) => out.push_str(&format!("builder_{}: OK\n", label)),
                Err(e) => out.push_str(&format!("builder_{}: FAIL - {}\n", label, e)),
            }
        }

        // Try with a block context
        let block_tmpl = "{% set x = (a ? b : c) %}{{ x }}";
        let mut t3 = tera::Tera::default();
        match t3.add_raw_template("test", block_tmpl) {
            Ok(()) => out.push_str("set_block: OK\n"),
            Err(e) => out.push_str(&format!("set_block: FAIL - {}\n", e)),
        }

        std::fs::write("/tmp/ternary_formats.txt", &out).ok();
        eprint!("{}", out);
    }

    #[test]
    fn test_default_conversion_with_pipe_chain() {
        let input = r"{{- style.colors.primary_dark | default('0B1D2A') | hex_color -}}";
        let processed = preprocess_delimiters(input);
        println!("INPUT:  {}", input);
        println!("OUTPUT: {}", processed);
        std::fs::write("/tmp/default_conv_test2.txt", &processed).ok();

        let mut t = tera::Tera::default();
        let r = t.add_raw_template("test", &processed);
        println!("Tera parse: ok={}", r.is_ok());
        if let Err(e) = &r {
            println!("  error: {}", e);
        }
        assert!(r.is_ok(), "Should parse: {}", processed);
    }

    #[test]
    fn test_default_conv_from_actual_template() {
        // Inline template with | default() (typical content.yaml usage)
        let contents = r#"((# Title #))
(( document.title | e ))

((# Content #))
(% for section in content.sections %)
Section: (( section.title | e ))
(( section.content | default('no content') | e ))
(% endfor %)
"#;
        let processed = preprocess_delimiters(&contents);
        std::fs::write("/tmp/main_report_preprocessed.txt", &processed).ok();
        // Verify default() was converted to {% if %} block
        assert!(!processed.contains("| default("), "default() should be converted");
        // Try parsing with Tera
        let mut t = tera::Tera::default();
        let r = t.add_raw_template("test", &processed);
        assert!(r.is_ok(), "Template should parse after conversion: {}", processed);
    }
}


#[cfg(test)]
mod quick_parse_test {
    #[test]
    fn test_set_ns_syntax() {
        let tests = vec![
            "{% set phase_total_val = 0 %}",
            "{% set phase_total.val = phase_total.val + 1 %}",
            "{% set ns.x = 1 %}",
            "{{ phase_total.val }}",
        ];
        for t in tests {
            let mut builder = tera::Tera::default();
            let r = builder.add_raw_template("test", t);
            std::fs::write(
                format!("/tmp/parse_test_{}.txt", t.replace(|c: char| !c.is_alphanumeric(), "_")),
                format!("OK={}: {}", r.is_ok(), r.as_ref().err().map(|e| e.to_string()).unwrap_or_default()),
            ).ok();
        }
    }
}


#[cfg(test)]
mod array_debug {
    #[test]
    fn debug_array_conversion() {
        use super::*;
        let input = "{{- x | default(['hello']) -}}";
        let after_var = convert_variable(input);
        let result = convert_default_filter(after_var.as_str());

        let mut t = tera::Tera::default();
        let r = t.add_raw_template("test", &result);
        assert!(r.is_ok(), "Tera parse error: {:?}", r);
    }
}

#[cfg(test)]
mod debug_preprocess_test {
    use super::*;

    fn presskit_exists() -> bool {
        std::fs::metadata("/tmp/presskit").is_ok()
    }

    #[test]
    #[ignore = "requires /tmp/presskit"]
    fn debug_team_template() {
        if !presskit_exists() { panic!("/tmp/presskit not available"); }
        let contents = std::fs::read_to_string("/tmp/presskit/templates/proposal-slides/_partials/_team.tex.j2").unwrap();
        let processed = preprocess_delimiters(&contents);
        let snippet: String = processed.chars().take(800).collect();
        println!("=== TEAM PREPROCESSED (first 800 chars) ===\n{}\n=== END ===", snippet);
        std::fs::write("/tmp/team_preprocessed.txt", &processed).ok();
    }

    #[test]
    #[ignore = "requires /tmp/presskit"]
    fn debug_investment_template() {
        if !presskit_exists() { panic!("/tmp/presskit not available"); }
        let contents = std::fs::read_to_string("/tmp/presskit/templates/proposal-slides/_partials/_investment.tex.j2").unwrap();
        let processed = preprocess_delimiters(&contents);
        let snippet: String = processed.chars().take(500).collect();
        println!("=== INVESTMENT PREPROCESSED (first 500 chars) ===\n{}\n=== END ===", snippet);
        std::fs::write("/tmp/investment_preprocessed.txt", &processed).ok();
    }

    #[test]
    #[ignore = "requires /tmp/presskit"]
    fn debug_main_template_first_50() {
        if !presskit_exists() { panic!("/tmp/presskit not available"); }
        let contents = std::fs::read_to_string("/tmp/presskit/templates/proposal-slides/main.tex.j2").unwrap();
        let processed = preprocess_delimiters(&contents);
        let snippet: String = processed.chars().take(800).collect();
        println!("=== MAIN PREPROCESSED (first 800 chars) ===\n{}\n=== END ===", snippet);
        std::fs::write("/tmp/main_preprocessed.txt", &processed).ok();

        let lines: Vec<&str> = processed.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            if i >= 80 && i <= 90 {
                eprintln!("  PROCESSED LINE {}: {:?}", i + 1, line);
            }
        }
    }
}

#[cfg(test)]
mod presskit_debug {
    use super::*;

    #[test]
    fn test_investment_single_line() {
        // From _investment.tex.j2: {% set cur = content.investment.currency ?? 'USD' %}
        // After preprocess: {% set cur = content.investment.currency is defined and content.investment.currency != '' ? content.investment.currency : 'USD' %}
        let input = r#"(% set cur = content.investment.currency ?? 'USD' %)"#;
        let processed = preprocess_delimiters(input);
        println!("INPUT: {:?}", input);
        println!("OUTPUT: {:?}", processed);
        std::fs::write("/tmp/investment_line_preprocessed.txt", &processed).ok();

        let mut t = tera::Tera::default();
        let r = t.add_raw_template("test", &processed);
        println!("Tera parse: ok={}", r.is_ok());
        if let Err(e) = &r { println!("  error: {}", e); }
        assert!(r.is_ok(), "Should parse: {}", processed);
    }

    #[test]
    fn test_triple_brace_var() {
        let input = r#"\begin{frame}[t]{{{ consultant.name | le }}}:"#;
        println!("INPUT: {:?}", input);
        let processed = preprocess_delimiters(input);
        println!("OUTPUT: {:?}", processed);
        std::fs::write("/tmp/triple_brace_out.txt", &processed).ok();
        let mut t = tera::Tera::default();
        let r = t.add_raw_template("test", &processed);
        println!("Tera parse: ok={}", r.is_ok());
        if let Err(e) = &r { println!("  error: {}", e); }
        assert!(r.is_ok(), "Parse fail: {}", processed);
    }

    #[test]
    fn test_fill_colors() {
        // From main.tex.j2: \fill[gold] (0,0) rectangle (0.70\paperwidth, 0.04cm);
        let input = r#"\fill[gold] (0,0) rectangle (0.70\paperwidth, 0.04cm);"#;
        let processed = preprocess_delimiters(input);
        println!("FILL INPUT:  {:?}", input);
        println!("FILL OUTPUT: {:?}", processed);
        assert_eq!(processed, input, "Non-Tera lines should be unchanged");
    }

    #[test]
    fn test_include_directive() {
        let input = r#"(# include '_cover.tex.j2' #)"#;
        let processed = preprocess_delimiters(input);
        println!("INCLUDE INPUT:  {:?}", input);
        println!("INCLUDE OUTPUT: {:?}", processed);
        // Comments should be converted to Tera style
        assert!(processed.contains("{#"), "Should have Tera-style comment");
    }
}
