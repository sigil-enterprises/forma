use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "forma", about = "Schema-agnostic document rendering framework")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(long, global = true, default_value = "error", action = clap::ArgAction::Set)]
    log_level: String,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a project's documents against their schemas
    Validate {
        /// Path to the project directory
        #[arg()]
        project_dir: PathBuf,

        /// Fail on warnings
        #[arg(long)]
        strict: bool,
    },
    /// Render a project to PDF/HTML/LaTeX
    Render {
        /// Path to the project directory
        #[arg()]
        project_dir: PathBuf,

        /// Template name to use
        #[arg(short, long)]
        template: Option<String>,
    },
    /// Validate a mapping file
    Mapping {
        #[command(subcommand)]
        sub: MappingCommand,
    },
    /// Compose content from notes
    Compose {
        #[command(subcommand)]
        sub: ComposeCommand,
    },
    /// Export embedded schemas
    Schema {
        #[command(subcommand)]
        sub: SchemaCommand,
    },
    /// List available templates
    Template {
        /// Search directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
    /// Initialize a new forma project
    Init {
        /// Client name
        #[arg()]
        client_name: String,

        /// Output directory
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum MappingCommand {
    Validate {
        /// Path to the project directory
        #[arg()]
        project_dir: PathBuf,

        /// Specific mapping file to validate
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ComposeCommand {
    Fill {
        /// Path to the project directory
        #[arg()]
        project_dir: PathBuf,

        /// Path to notes file
        #[arg(short, long)]
        notes: Option<PathBuf>,

        /// Model name
        #[arg(short, long, default_value = "claude-opus-4-6")]
        model: String,

        /// Max tokens
        #[arg(long, default_value_t = 8192)]
        max_tokens: u32,

        /// Show result without writing
        #[arg(long)]
        dry_run: bool,

        /// Overwrite existing content
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Subcommand)]
enum SchemaCommand {
    Export {
        /// Output directory for schemas
        #[arg(short, long, default_value = "schema")]
        dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    // Simple tracing setup
    let max_level = match cli.log_level.as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::ERROR,
    };
    tracing_subscriber::fmt()
        .with_max_level(max_level)
        .with_writer(std::io::stderr)
        .init();

    let result = match cli.command {
        Command::Validate { project_dir, strict } => cmd_validate(project_dir, strict),
        Command::Render { project_dir, template } => cmd_render(project_dir, template.as_deref()),
        Command::Mapping { sub } => match sub {
            MappingCommand::Validate { project_dir, file } => cmd_mapping_validate(project_dir, file),
        },
        Command::Compose { sub } => match sub {
            ComposeCommand::Fill { project_dir, notes, model, max_tokens, dry_run, overwrite } => {
                cmd_compose_fill(project_dir, notes, &model, max_tokens, dry_run, overwrite)
            }
        },
        Command::Schema { sub } => match sub {
            SchemaCommand::Export { dir } => cmd_schema_export(dir),
        },
        Command::Template { dir } => cmd_template_list(dir),
        Command::Init { client_name, dir } => cmd_init(client_name, dir),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

type CliResult = Result<(), String>;

fn yaml_to_json(val: &serde_yaml::Value) -> serde_json::Value {
    match val {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| serde_json::Number::from(0)))
            } else {
                serde_json::Value::Number(serde_json::Number::from(0))
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter_map(|(k, v)| {
                    k.as_str().map(|s| (s.to_string(), yaml_to_json(v)))
                })
                .collect();
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
    }
}

fn cmd_validate(project_dir: PathBuf, strict: bool) -> CliResult {
    let project_dir = project_dir.canonicalize().unwrap_or(project_dir);
    let result = forma_core::validate_project(&project_dir);

    for warning in &result.warnings {
        eprintln!("WARNING: {warning}");
    }
    for error in &result.errors {
        eprintln!("ERROR: {error}");
    }

    if !result.ok() || (strict && !result.warnings.is_empty()) {
        Err("Validation failed".into())
    } else {
        println!("OK: all documents valid");
        Ok(())
    }
}

fn cmd_render(project_dir: PathBuf, template: Option<&str>) -> CliResult {
    let project_dir = project_dir.canonicalize().unwrap_or(project_dir);
    let config = forma_core::FormaConfig::from_yaml(&project_dir.join("forma.yaml"))
        .map_err(|e| format!("Failed to load forma.yaml: {e}"))?;

    let style_path = config.resolve_style_path(&project_dir);
    let style = forma_core::load_style(&style_path);

    let output_dir = config.resolve_output_dir(&project_dir);
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create output dir: {e}"))?;

    let templates: Vec<(&String, &forma_core::TemplateEntry)> = if let Some(name) = template {
        config.templates.iter()
            .filter(|(k, _)| *k == name)
            .collect()
    } else {
        config.templates.iter().collect()
    };

    if templates.is_empty() {
        return Err("No templates found in forma.yaml".into());
    }

    for (name, entry) in &templates {
        let mapping_path = project_dir.join(&entry.mapping);
        if !mapping_path.exists() {
            eprintln!("SKIP: mapping file not found: {mapping_path:?}");
            continue;
        }

        let document = forma_core::load_document(&mapping_path, &project_dir)
            .map_err(|e| format!("Failed to load mapping {name}: {e}"))?;

        let tpl_path = config.resolve_template_path(name, &project_dir);
        let out = output_dir.join(format!("{name}.pdf"));

        eprintln!("Rendering {name}...");
        let doc_json = yaml_to_json(&document);
        let style_json = yaml_to_json(&style);
        forma_render::render_template(&tpl_path, &doc_json, &style_json, &out, Some(&project_dir))
            .map_err(|e| format!("Render failed for {name}: {e}"))?;
        println!("Wrote: {out:?}");
    }

    Ok(())
}

fn cmd_mapping_validate(project_dir: PathBuf, file: Option<PathBuf>) -> CliResult {
    let file_ref = file.as_deref();
    let project_dir = project_dir.canonicalize().unwrap_or(project_dir);

    let mapping_names: Vec<String> = match file_ref {
        Some(f) => {
            let name = f.file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            vec![name]
        }
        None => vec!["slides.yaml".into(), "report.yaml".into(), "brief.yaml".into()],
    };

    let mut any_found = false;
    let mut any_failed = false;

    for name in &mapping_names {
        let path = project_dir.join(name);
        if !path.exists() {
            continue;
        }
        any_found = true;
        eprintln!("Validating {name}...");
        let result = forma_core::validate_file(&path, Some(&project_dir));
        for warning in &result.warnings {
            eprintln!("WARNING: {warning}");
        }
        for error in &result.errors {
            eprintln!("ERROR: {error}");
        }
        if !result.ok() {
            any_failed = true;
        }
    }

    if !any_found {
        return Err("No mapping files found in project directory".into());
    }
    if any_failed {
        Err("Mapping validation failed".into())
    } else {
        println!("OK: all mappings valid");
        Ok(())
    }
}

fn cmd_compose_fill(
    project_dir: PathBuf,
    notes_path: Option<PathBuf>,
    model: &str,
    max_tokens: u32,
    dry_run: bool,
    overwrite: bool,
) -> CliResult {
    let project_dir = project_dir.canonicalize().unwrap_or(project_dir);
    let notes_path = notes_path.ok_or("--notes is required")?;
    let notes_path_ref = notes_path.as_path();
    let notes = std::fs::read_to_string(notes_path_ref).map_err(|e| format!("Failed to read notes: {e}"))?;

    let existing_path = project_dir.join("content.yaml");
    if existing_path.exists() && !overwrite && !dry_run {
        eprint!("content.yaml already exists. Overwrite? [y/N] ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read input: {e}"))?;
        if !input.trim().starts_with('y') {
            eprintln!("Aborted.");
            return Ok(());
        }
    }

    let result = forma_composer::fill_from_notes(
        &notes,
        forma_composer::SchemaType::Proposal,
        model,
        max_tokens,
        Some(&existing_path),
    ).map_err(|e| format!("Compose failed: {e}"))?;

    if dry_run {
        println!("{}", result.raw_yaml);
        return Ok(());
    }

    std::fs::write(&existing_path, &result.raw_yaml)
        .map_err(|e| format!("Failed to write content.yaml: {e}"))?;
    println!("Wrote: {:?}", existing_path);
    Ok(())
}

fn cmd_schema_export(dir: PathBuf) -> CliResult {
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir: {e}"))?;

    for (name, content) in forma_schema::embedded::all() {
        let path = dir.join(name);
        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write {name}: {e}"))?;
        println!("Wrote: {:?}", path);
    }

    Ok(())
}

fn cmd_template_list(dir: Option<PathBuf>) -> CliResult {
    let search_dir = dir.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("tests")
            .join("fixtures")
            .join("templates")
    });

    let mut entries: Vec<_> = if search_dir.exists() {
        std::fs::read_dir(&search_dir)
            .map_err(|e| format!("Failed to read directory: {e}"))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect()
    } else {
        vec![]
    };
    entries.sort();

    if entries.is_empty() {
        eprintln!("No template directories found.");
        return Ok(());
    }

    // Print table header
    println!("{:<30} {:<10} {:<12} {}", "Name", "Format", "Engine", "Description");
    println!("{:-<68}", "");

    for entry in &entries {
        let manifest_path = entry.join("manifest.yaml");
        if !manifest_path.exists() {
            continue;
        }

        let manifest = match forma_render::TemplateManifest::from_path(entry) {
            Ok(m) => m,
            Err(_) => continue,
        };

        println!(
            "{:<30} {:<10} {:<12} {}",
            entry.file_name().map(|s| s.to_string_lossy()).unwrap_or_default(),
            manifest.format,
            manifest.engine,
            manifest.description.unwrap_or_default()
        );
    }

    Ok(())
}

fn cmd_init(client_name: String, dir: Option<PathBuf>) -> CliResult {
    let documents_dir = dir.unwrap_or_else(|| PathBuf::from("documents"));
    let slug = client_name.to_lowercase().replace(' ', "-");
    let project_dir = documents_dir.join(&slug);
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    // Determine template base relative path
    let up = "../..";

    let forma_config = serde_yaml::to_string(&serde_json::json!({
        "resourceType": "FormaConfig",
        "content": "content.yaml",
        "style": "style.yaml",
        "templates": {
            "slides": {
                "path": "path/to/proposal-slides-html",
                "mapping": "slides.yaml",
            },
            "report": {
                "path": "path/to/proposal-report",
                "mapping": "report.yaml",
            },
        },
        "output_dir": format!("{up}/var/builds/{slug}"),
        "publishing": {
            "google_drive_folder_id": "",
            "filename_prefix": slug.to_uppercase().chars().take(8).collect::<String>(),
        },
    })).map_err(|e| format!("Failed to serialize config: {e}"))?;

    std::fs::write(project_dir.join("forma.yaml"), forma_config)
        .map_err(|e| format!("Failed to write forma.yaml: {e}"))?;

    let content_yaml = format!(
        "resourceType: ProposalContent\n\n\
        # {client_name} - content.yaml\n\
        # Fill in your content here.\n\n\
        engagement:\n\
          title: \"{client_name}\"\n\
          subtitle: \"\"\n\
          reference: \"\"\n\
          date: \"\"\n\n\
        client:\n\
          name: \"{client_name}\"\n\n\
        executive_summary:\n\
          headline: \"\"\n\
          key_points: []\n",
    );
    std::fs::write(project_dir.join("content.yaml"), content_yaml)
        .map_err(|e| format!("Failed to write content.yaml: {e}"))?;

    let slides_yaml = "\
resourceType: SlideDocument\n\n\
slides:\n\
  - type: cover\n\
    title: !include \"@content.yaml:engagement.title\"\n\
    client: !include \"@content.yaml:client.name\"\n\n\
  - type: exec_summary\n\
    headline: !include \"@content.yaml:executive_summary.headline\"\n\
    key_points: !include \"@content.yaml:executive_summary.key_points\"\n\n\
  - type: closing\n\
    tagline: \"\"\n";
    std::fs::write(project_dir.join("slides.yaml"), slides_yaml)
        .map_err(|e| format!("Failed to write slides.yaml: {e}"))?;

    let report_yaml = "\
resourceType: ReportDocument\n\n\
meta:\n\
  title: !include \"@content.yaml:engagement.title\"\n\
  client: !include \"@content.yaml:client.name\"\n\n\
chapters:\n\
  - title: Executive Summary\n\
    sections:\n\
      - title: Overview\n\
        blocks:\n\
          - type: paragraph\n\
            text: !include \"@content.yaml:executive_summary.headline\"\n";
    std::fs::write(project_dir.join("report.yaml"), report_yaml)
        .map_err(|e| format!("Failed to write report.yaml: {e}"))?;

    println!("Created: {project_dir:?}/");
    println!("  Edit {project_dir:?}/content.yaml with your semantic content");
    println!("  Edit {project_dir:?}/slides.yaml to map content -> slides");
    println!("  Edit {project_dir:?}/report.yaml to map content -> report chapters");
    println!("  Run: forma render {project_dir:?} --template slides");

    Ok(())
}
