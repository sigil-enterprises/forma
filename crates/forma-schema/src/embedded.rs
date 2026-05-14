/// Embedded JSON schema YAML files, loaded at compile time.

pub const FORMA_CONFIG: &str = include_str!("../schema/forma-config.schema.yaml");
pub const PROPOSAL_CONTENT: &str = include_str!("../schema/proposal-content.schema.yaml");
pub const SLIDE_DOCUMENT: &str = include_str!("../schema/slide-document.schema.yaml");
pub const REPORT_DOCUMENT: &str = include_str!("../schema/report-document.schema.yaml");

/// All embedded schemas as [(name, yaml_content)].
pub fn all() -> &'static [(&'static str, &'static str)] {
    &[
        ("forma-config.schema.yaml", FORMA_CONFIG),
        ("proposal-content.schema.yaml", PROPOSAL_CONTENT),
        ("slide-document.schema.yaml", SLIDE_DOCUMENT),
        ("report-document.schema.yaml", REPORT_DOCUMENT),
    ]
}
