use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HtmlRenderError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("headless_chrome is required for HTML→PDF. Install Chromium and enable the 'pdf' feature.")]
    ChromeUnavailable,
}

pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn new() -> Self { Self }

    pub fn save_html(&self, html_source: &str, output_path: &Path) -> Result<(), HtmlRenderError> {
        output_path.parent().map(|p| std::fs::create_dir_all(p)).transpose()?;
        std::fs::write(output_path, html_source)?;
        Ok(())
    }

    pub fn render_pdf(
        &self,
        html_source: &str,
        output_path: &Path,
        _workdir: Option<&Path>,
    ) -> Result<(), HtmlRenderError> {
        #[cfg(feature = "pdf")]
        {
            use std::path::PathBuf;

            let effective_workdir = workdir.unwrap_or(output_path.parent().unwrap_or(Path("").as_ref()));
            std::fs::create_dir_all(effective_workdir)?;

            let html_file = effective_workdir.join("_forma_render.html");
            std::fs::write(&html_file, html_source)?;

            let file_url = format!("file://{}", html_file.display());

            let client = headless_chrome::Builder::new()
                .map_err(|_| HtmlRenderError::ChromeUnavailable)?
                .build()
                .map_err(|_| HtmlRenderError::ChromeUnavailable)?;

            let tab = client.get_static_tab()
                .ok_or(HtmlRenderError::ChromeUnavailable)?;
            tab.navigate_to(&file_url)
                .map_err(|_| HtmlRenderError::ChromeUnavailable)?;
            tab.eval("document.readyState", |val| {
                match val {
                    Ok(v) => v == "complete",
                    Err(_) => false,
                }
            });

            tab.pdf(headless_chrome::PDFOptions {
                path: Some(PathBuf::from(output_path)),
                scale: Some(1.0),
                paper_width: Some(1280.0 / 96.0),
                paper_height: Some(720.0 / 96.0),
                margin_options: headless_chrome::PDFMarginOptions {
                    top: Some(0.0),
                    right: Some(0.0),
                    bottom: Some(0.0),
                    left: Some(0.0),
                },
                ..Default::default()
            }).map_err(|_| HtmlRenderError::ChromeUnavailable)?;

            Ok(())
        }

        #[cfg(not(feature = "pdf"))]
        {
            self.save_html(html_source, &output_path.with_extension("html"))?;
            eprintln!("Note: HTML saved to {}. For PDF, enable 'pdf' feature.",
                output_path.with_extension("html").display());
            Ok(())
        }
    }
}
