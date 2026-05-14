use serde::{Deserialize, Serialize};

fn default_investment() -> String { "Investment".into() }
fn default_usd() -> String { "USD".into() }

// -- SlideDocument mapping types --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideDocument {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub slides: Vec<Slide>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Slide {
    #[serde(rename = "cover")]
    Cover(CoverSlide),
    #[serde(rename = "exec_summary")]
    ExecSummary(ExecSummarySlide),
    #[serde(rename = "section_divider")]
    SectionDivider(SectionDividerSlide),
    #[serde(rename = "pain_points")]
    PainPoints(PainPointsSlide),
    #[serde(rename = "metrics")]
    Metrics(MetricsSlide),
    #[serde(rename = "solution_pillars")]
    SolutionPillars(SolutionPillarsSlide),
    #[serde(rename = "differentiators")]
    Differentiators(DifferentiatorsSlide),
    #[serde(rename = "timeline")]
    Timeline(TimelineSlide),
    #[serde(rename = "investment_table")]
    InvestmentTable(InvestmentTableSlide),
    #[serde(rename = "team_grid")]
    TeamGrid(TeamGridSlide),
    #[serde(rename = "next_steps")]
    NextSteps(NextStepsSlide),
    #[serde(rename = "closing")]
    Closing(ClosingSlide),
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverSlide {
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    pub client: String,
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub logo: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecSummarySlide {
    pub headline: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub key_points: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionDividerSlide {
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitledItem {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PainPointsSlide {
    #[serde(default = "default_challenge")]
    pub title: String,
    pub items: Vec<TitledItem>,
}

fn default_challenge() -> String { "The Challenge".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideMetric {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsSlide {
    #[serde(default = "default_by_the_numbers")]
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub metrics: Vec<SlideMetric>,
}

fn default_by_the_numbers() -> String { "By the Numbers".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolutionPillarsSlide {
    #[serde(default = "default_our_approach")]
    pub title: String,
    #[serde(default)]
    pub overview: String,
    pub pillars: Vec<TitledItem>,
}

fn default_our_approach() -> String { "Our Approach".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DifferentiatorsSlide {
    #[serde(default = "default_why_us")]
    pub title: String,
    pub items: Vec<TitledItem>,
}

fn default_why_us() -> String { "Why Us".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideTimelinePhase {
    pub name: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub start_week: Option<i64>,
    #[serde(default)]
    pub end_week: Option<i64>,
    #[serde(default)]
    pub activities: Vec<String>,
    #[serde(default)]
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineSlide {
    #[serde(default = "default_project_roadmap")]
    pub title: String,
    pub phases: Vec<SlideTimelinePhase>,
}

fn default_project_roadmap() -> String { "Project Roadmap".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlideLineItem {
    pub service: String,
    #[serde(default)]
    pub quantity: f64,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub rate_usd: f64,
    #[serde(default)]
    pub total_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvestmentPhaseEntry {
    pub name: String,
    #[serde(default)]
    pub duration: String,
    pub line_items: Vec<SlideLineItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvestmentTableSlide {
    #[serde(default = "default_investment")]
    pub title: String,
    #[serde(default = "default_usd")]
    pub currency: String,
    #[serde(default)]
    pub notes: Vec<String>,
    pub phases: Vec<InvestmentPhaseEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamMember {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub credentials: String,
    #[serde(default)]
    pub experience_years: Option<i64>,
    #[serde(default)]
    pub education: Vec<String>,
    #[serde(default)]
    pub expertise: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamGridSlide {
    #[serde(default = "default_our_team")]
    pub title: String,
    pub consultants: Vec<TeamMember>,
    #[serde(default)]
    pub partners: Vec<ReportPartner>,
}

fn default_our_team() -> String { "Our Team".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NextStepsSlide {
    #[serde(default = "default_next_steps")]
    pub title: String,
    #[serde(default)]
    pub intro: String,
    pub steps: Vec<TitledItem>,
}

fn default_next_steps() -> String { "Next Steps".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClosingSlide {
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub logo: String,
}

// -- ReportDocument mapping types --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportDocument {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(default)]
    pub meta: Option<ReportMeta>,
    #[serde(default)]
    pub cover: Option<ReportCover>,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportMeta {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub client: String,
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub confidentiality: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportCover {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub client: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub contact_name: String,
    #[serde(default)]
    pub contact_email: String,
    #[serde(default)]
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    #[serde(default)]
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    #[serde(default)]
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Block {
    #[serde(rename = "paragraph")]
    Paragraph(ParagraphBlock),
    #[serde(rename = "heading")]
    Heading(HeadingBlock),
    #[serde(rename = "bullet_list")]
    BulletList(BulletListBlock),
    #[serde(rename = "table")]
    Table(TableBlock),
    #[serde(rename = "metrics_table")]
    MetricsTable(MetricsTableBlock),
    #[serde(rename = "investment_table")]
    InvestmentTable(InvestmentTableBlock),
    #[serde(rename = "timeline")]
    TimelineBlock(TimelineBlock),
    #[serde(rename = "team")]
    Team(TeamBlock),
    #[serde(rename = "figure")]
    Figure(FigureBlock),
    #[serde(rename = "raw_latex")]
    RawLatex(RawLatexBlock),
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphBlock {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeadingBlock {
    pub text: String,
    #[serde(default = "default_h3")]
    pub level: u32,
}

fn default_h3() -> u32 { 3 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulletListBlock {
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableBlock {
    #[serde(default)]
    pub caption: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsTableBlock {
    #[serde(default)]
    pub caption: String,
    pub metrics: Vec<SlideMetric>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvestmentTableBlock {
    #[serde(default = "default_usd")]
    pub currency: String,
    #[serde(default)]
    pub notes: Vec<String>,
    pub phases: Vec<ReportInvestmentPhase>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportInvestmentPhase {
    pub name: String,
    #[serde(default)]
    pub duration: String,
    pub line_items: Vec<ReportLineItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportLineItem {
    pub service: String,
    #[serde(default)]
    pub quantity: f64,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub rate_usd: f64,
    #[serde(default)]
    pub total_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineBlock {
    pub phases: Vec<SlideTimelinePhase>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamBlock {
    pub consultants: Vec<TeamMember>,
    #[serde(default)]
    pub partners: Vec<ReportPartner>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportPartner {
    pub name: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigureBlock {
    pub path: String,
    #[serde(default)]
    pub caption: String,
    #[serde(default = "default_linewidth")]
    pub width: String,
}

fn default_linewidth() -> String { "\\linewidth".into() }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawLatexBlock {
    pub latex: String,
}
