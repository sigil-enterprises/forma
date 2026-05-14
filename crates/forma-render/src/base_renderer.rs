use std::env;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("No PDF produced by {0}. Check the log for errors.")]
    NoPdf(String),
    #[error("LaTeX compilation failed (exit {0}): {1}")]
    CompileFailed(i32, String),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("No LaTeX engine found: {0}")]
    EngineNotFound(String),
}

#[derive(Debug)]
pub struct BaseRenderer {
    engine: &'static str,
}

impl BaseRenderer {
    pub fn new(engine: &'static str) -> Self {
        Self { engine }
    }

    pub fn render(
        &self,
        tex_source: &str,
        output_path: &Path,
        project_dir: Option<&Path>,
        fonts_dirs: Option<&[std::path::PathBuf]>,
    ) -> Result<(), RenderError> {
        output_path.parent().map(|p| std::fs::create_dir_all(p)).transpose()?;

        let tmp = TempDir::new()?;
        let tex_file = tmp.path().join("document.tex");
        std::fs::write(&tex_file, tex_source)?;

        for _ in 0..2 {
            self._compile(&tex_file, tmp.path(), project_dir, fonts_dirs)?;
        }

        let pdf_file = tmp.path().join("document.pdf");
        if !pdf_file.exists() {
            return Err(RenderError::NoPdf(self.engine.to_string()));
        }

        std::fs::copy(&pdf_file, output_path)?;
        Ok(())
    }

    fn _compile(
        &self,
        tex_file: &Path,
        workdir: &Path,
        project_dir: Option<&Path>,
        fonts_dirs: Option<&[std::path::PathBuf]>,
    ) -> Result<(), RenderError> {
        let cmd = [
            self.engine,
            "-interaction=nonstopmode",
            &format!("-output-directory={}", workdir.display()),
            tex_file.to_str().unwrap(),
        ];

        let mut env = env::vars_os().collect::<Vec<_>>();

        if let Some(pd) = project_dir {
            let val = format!("{}//:", pd.display());
            let existing = env.iter().find(|(k, _)| k == "TEXINPUTS").map(|(_, v)| v.clone());
            let new_val = match existing {
                Some(e) => format!("{}:{}", val, e.to_string_lossy()),
                None => format!("{}:", val),
            };
            env.push(("TEXINPUTS".into(), new_val.into()));
        }

        if let Some(fonts) = fonts_dirs {
            if !fonts.is_empty() {
                let extra: String = fonts.iter()
                    .map(|d| format!("{}//:", d.display()))
                    .collect::<Vec<_>>()
                    .join(":");
                let existing_tex = env.iter().find(|(k, _)| k == "TEXINPUTS").map(|(_, v)| v.clone());
                let new_tex = match existing_tex {
                    Some(e) => format!("{}:{}", extra, e.to_string_lossy()),
                    None => format!("{}:", extra),
                };
                env.push(("TEXINPUTS".into(), new_tex.into()));

                let existing_os = env.iter().find(|(k, _)| k == "OSFONTDIR").map(|(_, v)| v.clone());
                let new_os = match existing_os {
                    Some(e) => format!("{}:{}", extra, e.to_string_lossy()),
                    None => extra,
                };
                env.push(("OSFONTDIR".into(), new_os.into()));
            }
        }

        let output = Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(workdir)
            .envs(env)
            .output()?;

        let pdf_produced = workdir.join("document.pdf").exists();
        if !pdf_produced {
            let log_path = workdir.join("document.log");
            let log_content = if log_path.exists() {
                let log = std::fs::read_to_string(&log_path).unwrap_or_default();
                let lines: Vec<&str> = log.lines().collect();
                let start = lines.len().saturating_sub(60);
                lines[start..].join("\n")
            } else {
                String::from_utf8_lossy(&output.stdout).to_string()
            };
            if output.status.code() == Some(0) || output.status.success() {
                return Ok(());
            }
            return Err(RenderError::CompileFailed(
                output.status.code().unwrap_or(-1),
                log_content,
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct XelatexRenderer(pub(crate) BaseRenderer);
#[derive(Debug)]
pub struct PdflatexRenderer(pub(crate) BaseRenderer);
#[derive(Debug)]
pub struct LualatexRenderer(pub(crate) BaseRenderer);

impl XelatexRenderer {
    pub fn new() -> Self { Self(BaseRenderer::new("xelatex")) }
}
impl PdflatexRenderer {
    pub fn new() -> Self { Self(BaseRenderer::new("pdflatex")) }
}
impl LualatexRenderer {
    pub fn new() -> Self { Self(BaseRenderer::new("lualatex")) }
}

impl std::fmt::Display for XelatexRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "xelatex") }
}
impl std::fmt::Display for PdflatexRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "pdflatex") }
}
impl std::fmt::Display for LualatexRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "lualatex") }
}
