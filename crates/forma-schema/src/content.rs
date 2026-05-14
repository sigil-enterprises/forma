use serde::{Deserialize, Serialize};

// -- Publishing / style primitives (also used by forma-core) --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PublishingConfig {
    #[serde(default)]
    pub google_drive_folder_id: String,
    #[serde(default)]
    pub filename_prefix: String,
}

impl Default for PublishingConfig {
    fn default() -> Self {
        Self {
            google_drive_folder_id: String::new(),
            filename_prefix: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BrandConfig {
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub logo_white: String,
}

impl Default for BrandConfig {
    fn default() -> Self {
        Self {
            logo: String::new(),
            logo_white: String::new(),
        }
    }
}

fn default_primary_dark() -> String { "#0B1D2A".into() }
fn default_primary_accent() -> String { "#F58220".into() }
fn default_white() -> String { "#FFFFFF".into() }
fn default_gray_light() -> String { "#F5F5F5".into() }
fn default_gray_medium() -> String { "#E0E0E0".into() }
fn default_gray_dark() -> String { "#666666".into() }
fn default_text_primary() -> String { "#333333".into() }
fn default_text_secondary() -> String { "#666666".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ColorConfig {
    #[serde(default = "default_primary_dark")]
    pub primary_dark: String,
    #[serde(default = "default_primary_accent")]
    pub primary_accent: String,
    #[serde(default = "default_white")]
    pub white: String,
    #[serde(default = "default_gray_light")]
    pub gray_light: String,
    #[serde(default = "default_gray_medium")]
    pub gray_medium: String,
    #[serde(default = "default_gray_dark")]
    pub gray_dark: String,
    #[serde(default = "default_text_primary")]
    pub text_primary: String,
    #[serde(default = "default_text_secondary")]
    pub text_secondary: String,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            primary_dark: default_primary_dark(),
            primary_accent: default_primary_accent(),
            white: default_white(),
            gray_light: default_gray_light(),
            gray_medium: default_gray_medium(),
            gray_dark: default_gray_dark(),
            text_primary: default_text_primary(),
            text_secondary: default_text_secondary(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TypographyConfig {
    #[serde(default = "default_font_primary")]
    pub font_primary: String,
    #[serde(default = "default_font_secondary")]
    pub font_secondary: String,
    #[serde(default = "default_font_mono")]
    pub font_mono: String,
    #[serde(default)]
    pub sizes: std::collections::HashMap<String, u32>,
}

fn default_font_primary() -> String { "Helvetica".into() }
fn default_font_secondary() -> String { "Helvetica".into() }
fn default_font_mono() -> String { "Courier".into() }

impl Default for TypographyConfig {
    fn default() -> Self {
        Self {
            font_primary: default_font_primary(),
            font_secondary: default_font_secondary(),
            font_mono: default_font_mono(),
            sizes: [
                ("xs".into(), 9),
                ("sm".into(), 10),
                ("base".into(), 11),
                ("md".into(), 12),
                ("lg".into(), 14),
                ("xl".into(), 16),
                ("xl2".into(), 20),
                ("xl3".into(), 24),
                ("xl4".into(), 32),
            ]
            .into_iter()
            .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LayoutConfig {
    #[serde(default = "default_page_size")]
    pub page_size: String,
    #[serde(default = "default_slides_aspect_ratio")]
    pub slides_aspect_ratio: String,
}

fn default_page_size() -> String { "a4".into() }
fn default_slides_aspect_ratio() -> String { "169".into() }

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            page_size: default_page_size(),
            slides_aspect_ratio: default_slides_aspect_ratio(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FormaStyle {
    #[serde(default)]
    pub brand: BrandConfig,
    #[serde(default)]
    pub colors: ColorConfig,
    #[serde(default)]
    pub typography: TypographyConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub publishing: PublishingConfig,
}

impl Default for FormaStyle {
    fn default() -> Self {
        Self {
            brand: Default::default(),
            colors: Default::default(),
            typography: Default::default(),
            layout: Default::default(),
            publishing: Default::default(),
        }
    }
}

// -- ProposalContent --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Engagement {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub reference: String,
    pub date: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub confidentiality: String,
    #[serde(default = "default_en")]
    pub language: String,
}

fn default_version() -> String { "1.0".into() }
fn default_en() -> String { "en".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ClientContact {
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Client {
    pub name: String,
    #[serde(default)]
    pub industry: String,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub contact: Option<ClientContact>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExecutiveSummary {
    pub headline: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub key_points: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PainPoint {
    pub title: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Metric {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CurrentState {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Context {
    #[serde(default)]
    pub problem_statement: String,
    #[serde(default)]
    pub pain_points: Vec<PainPoint>,
    #[serde(default)]
    pub current_state: Option<CurrentState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Pillar {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Differentiator {
    pub title: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Solution {
    #[serde(default)]
    pub overview: String,
    #[serde(default)]
    pub pillars: Vec<Pillar>,
    #[serde(default)]
    pub differentiators: Vec<Differentiator>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Phase {
    pub name: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub activities: Vec<String>,
    #[serde(default)]
    pub deliverables: Vec<String>,
    #[serde(default)]
    pub start_week: Option<i32>,
    #[serde(default)]
    pub end_week: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Timeline {
    #[serde(default)]
    pub phases: Vec<Phase>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LineItem {
    pub service: String,
    #[serde(default = "default_one")]
    pub quantity: f64,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub rate_usd: f64,
    #[serde(default)]
    pub total_usd: f64,
}

fn default_one() -> f64 { 1.0 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct InvestmentPhase {
    pub name: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub line_items: Vec<LineItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Investment {
    #[serde(default = "default_usd")]
    pub currency: String,
    #[serde(default)]
    pub secondary_currency: String,
    #[serde(default = "default_one")]
    pub exchange_rate: f64,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub phases: Vec<InvestmentPhase>,
}

fn default_usd() -> String { "USD".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Consultant {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub credentials: String,
    #[serde(default)]
    pub experience_years: Option<i32>,
    #[serde(default)]
    pub education: Vec<String>,
    #[serde(default)]
    pub expertise: Vec<String>,
    #[serde(default)]
    pub photo: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Partner {
    pub name: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Team {
    #[serde(default)]
    pub consultants: Vec<Consultant>,
    #[serde(default)]
    pub partners: Vec<Partner>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Step {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct NextSteps {
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Closing {
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub logo: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProposalContent {
    pub engagement: Engagement,
    pub client: Client,
    pub executive_summary: ExecutiveSummary,
    #[serde(default)]
    pub context: Option<Context>,
    #[serde(default)]
    pub solution: Option<Solution>,
    #[serde(default)]
    pub timeline: Option<Timeline>,
    #[serde(default)]
    pub investment: Option<Investment>,
    #[serde(default)]
    pub team: Option<Team>,
    #[serde(default)]
    pub next_steps: Option<NextSteps>,
    #[serde(default)]
    pub closing: Option<Closing>,
    #[serde(default)]
    pub publishing: PublishingConfig,
}

// -- BriefContent --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BriefMeta {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    pub date: String,
    #[serde(default)]
    pub prepared_for: String,
    #[serde(default)]
    pub prepared_by: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BriefSection {
    pub heading: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BriefContent {
    pub meta: BriefMeta,
    #[serde(default)]
    pub sections: Vec<BriefSection>,
    #[serde(default)]
    pub call_to_action: String,
    #[serde(default)]
    pub contact_email: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub publishing: PublishingConfig,
}

// -- CaseStudyContent --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CaseStudyMeta {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    pub client_name: String,
    #[serde(default)]
    pub industry: String,
    pub date: String,
    #[serde(default = "default_public")]
    pub confidentiality: String,
}

fn default_public() -> String { "public".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Challenge {
    pub statement: String,
    #[serde(default)]
    pub details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Approach {
    pub overview: String,
    #[serde(default)]
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Outcome {
    pub headline: String,
    #[serde(default)]
    pub results: Vec<String>,
    #[serde(default)]
    pub quote: String,
    #[serde(default)]
    pub quote_attribution: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CaseStudyContent {
    pub meta: CaseStudyMeta,
    pub challenge: Challenge,
    pub approach: Approach,
    pub outcomes: Outcome,
    #[serde(default)]
    pub technologies: Vec<String>,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub hero_image: String,
    #[serde(default)]
    pub publishing: PublishingConfig,
}

// -- ContentType enum for composer --

/// Available content types that can be composed via the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Proposal,
    Brief,
    CaseStudy,
    StatusReport,
}

impl ContentType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Proposal => "ProposalContent",
            Self::Brief => "BriefContent",
            Self::CaseStudy => "CaseStudyContent",
            Self::StatusReport => "StatusReportContent",
        }
    }

    /// Return the JSON Schema for this content type as an indented JSON string.
    pub fn json_schema_str(&self) -> String {
        match self {
            Self::Proposal => {
                let schema = schemars::schema_for!(ProposalContent);
                serde_json::to_string_pretty(&schema).unwrap()
            }
            Self::Brief => {
                let schema = schemars::schema_for!(BriefContent);
                serde_json::to_string_pretty(&schema).unwrap()
            }
            Self::CaseStudy => {
                let schema = schemars::schema_for!(CaseStudyContent);
                serde_json::to_string_pretty(&schema).unwrap()
            }
            Self::StatusReport => {
                let schema = schemars::schema_for!(StatusReportContent);
                serde_json::to_string_pretty(&schema).unwrap()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProjectInfo {
    pub name: String,
    pub client: String,
    pub period_start: String,
    pub period_end: String,
    #[serde(default = "default_green")]
    pub overall_rag: String,
    #[serde(default)]
    pub phase: String,
}

fn default_green() -> String { "green".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Milestone {
    pub name: String,
    pub due_date: String,
    #[serde(default = "default_on_track")]
    pub status: String,
    #[serde(default)]
    pub notes: String,
}

fn default_on_track() -> String { "on-track".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Budget {
    pub planned_usd: f64,
    pub actual_usd: f64,
    pub forecast_usd: f64,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Risk {
    pub description: String,
    #[serde(default = "default_medium")]
    pub severity: String,
    pub mitigation: String,
    #[serde(default)]
    pub owner: String,
}

fn default_medium() -> String { "medium".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StatusAction {
    #[serde(rename = "action")]
    pub action_text: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub due_date: String,
    #[serde(default = "default_pending")]
    pub status: String,
}

fn default_pending() -> String { "pending".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StatusReportContent {
    pub project: ProjectInfo,
    pub summary: String,
    #[serde(default)]
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub budget: Option<Budget>,
    #[serde(default)]
    pub risks: Vec<Risk>,
    #[serde(default)]
    pub actions: Vec<StatusAction>,
    #[serde(default)]
    pub next_period_focus: String,
    #[serde(default)]
    pub publishing: PublishingConfig,
}
