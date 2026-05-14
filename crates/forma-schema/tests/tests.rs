use forma_schema::{
    content::{
        BriefContent, BriefMeta, CaseStudyContent, CaseStudyMeta, Challenge, Approach, Outcome,
        Engagement, Client, ExecutiveSummary, ProposalContent, ProjectInfo, StatusReportContent,
        ColorConfig, TypographyConfig, LayoutConfig, FormaStyle, ContentType,
    },
    document::{SlideDocument, ReportDocument, CoverSlide, Chapter, Slide},
};

// -- Embedded schema tests --

#[test]
fn all_schemas_returned() {
    let schemas = forma_schema::embedded::all();
    assert_eq!(schemas.len(), 4);
    let names: Vec<&str> = schemas.iter().map(|(name, _)| *name).collect();
    assert!(names.contains(&"forma-config.schema.yaml"));
    assert!(names.contains(&"proposal-content.schema.yaml"));
    assert!(names.contains(&"slide-document.schema.yaml"));
    assert!(names.contains(&"report-document.schema.yaml"));
}

#[test]
fn schema_content_nonempty() {
    let schemas = forma_schema::embedded::all();
    for (name, content) in schemas {
        assert!(!content.is_empty(), "{} should not be empty", name);
        assert!(content.contains("$schema") || content.contains("\"type\": \"object\""),
            "{} should be valid JSON Schema", name);
    }
}

// -- FormaStyle default tests --

#[test]
fn color_config_defaults() {
    let colors = ColorConfig::default();
    assert_eq!(colors.primary_dark, "#0B1D2A");
    assert_eq!(colors.primary_accent, "#F58220");
    assert_eq!(colors.white, "#FFFFFF");
    assert_eq!(colors.gray_light, "#F5F5F5");
    assert_eq!(colors.gray_medium, "#E0E0E0");
    assert_eq!(colors.gray_dark, "#666666");
    assert_eq!(colors.text_primary, "#333333");
    assert_eq!(colors.text_secondary, "#666666");
}

#[test]
fn typography_config_defaults() {
    let typos = TypographyConfig::default();
    assert_eq!(typos.font_primary, "Helvetica");
    assert_eq!(typos.font_secondary, "Helvetica");
    assert_eq!(typos.font_mono, "Courier");
    assert!(typos.sizes.contains_key("xs"));
    assert_eq!(typos.sizes.get("xs"), Some(&9));
    assert_eq!(typos.sizes.get("xl4"), Some(&32));
}

#[test]
fn layout_config_defaults() {
    let layout = LayoutConfig::default();
    assert_eq!(layout.page_size, "a4");
    assert_eq!(layout.slides_aspect_ratio, "169");
}

#[test]
fn forma_style_defaults() {
    let style = FormaStyle::default();
    assert_eq!(style.brand.logo, "");
    assert_eq!(style.colors.primary_dark, "#0B1D2A");
    assert_eq!(style.typography.font_primary, "Helvetica");
    assert_eq!(style.layout.page_size, "a4");
    assert_eq!(style.publishing.google_drive_folder_id, "");
}

// -- Content serialization tests --

#[test]
fn proposal_content_roundtrip() {
    let content = ProposalContent {
        engagement: Engagement {
            title: "Test Engagement".into(),
            subtitle: "Test Subtitle".into(),
            reference: "REF-001".into(),
            date: "2026-01-01".into(),
            version: "1.0".into(),
            confidentiality: "Confidential".into(),
            language: "en".into(),
        },
        client: Client {
            name: "Test Client".into(),
            industry: "Tech".into(),
            size: "Enterprise".into(),
            contact: None,
        },
        executive_summary: ExecutiveSummary {
            headline: "Test Headline".into(),
            body: "Test body text.".into(),
            key_points: vec!["Point 1".into(), "Point 2".into()],
        },
        context: None,
        solution: None,
        timeline: None,
        investment: None,
        team: None,
        next_steps: None,
        closing: None,
        publishing: Default::default(),
    };

    let yaml = serde_yaml::to_string(&content).unwrap();
    let decoded: ProposalContent = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(decoded.engagement.title, content.engagement.title);
    assert_eq!(decoded.executive_summary.headline, content.executive_summary.headline);
}

#[test]
fn brief_content_roundtrip() {
    let content = BriefContent {
        meta: BriefMeta {
            title: "Test Brief".into(),
            subtitle: "Test Subtitle".into(),
            date: "2026-01-01".into(),
            prepared_for: "Client".into(),
            prepared_by: "Author".into(),
        },
        sections: vec![],
        call_to_action: "Contact us".into(),
        contact_email: "test@example.com".into(),
        logo: "logo.png".into(),
        publishing: Default::default(),
    };

    let yaml = serde_yaml::to_string(&content).unwrap();
    let decoded: BriefContent = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(decoded.meta.title, content.meta.title);
}

#[test]
fn case_study_content_roundtrip() {
    let content = CaseStudyContent {
        meta: CaseStudyMeta {
            title: "Test Study".into(),
            subtitle: "Subtitle".into(),
            client_name: "Client".into(),
            industry: "Industry".into(),
            date: "2026-01-01".into(),
            confidentiality: "Public".into(),
        },
        challenge: Challenge {
            statement: "The challenge".into(),
            details: vec!["Detail 1".into()],
        },
        approach: Approach {
            overview: "The approach".into(),
            steps: vec!["Step 1".into()],
        },
        outcomes: Outcome {
            headline: "Success".into(),
            results: vec!["Result 1".into()],
            quote: "Great work!".into(),
            quote_attribution: "CEO".into(),
        },
        technologies: vec!["tech1".into()],
        logo: "logo.png".into(),
        hero_image: "hero.png".into(),
        publishing: Default::default(),
    };

    let yaml = serde_yaml::to_string(&content).unwrap();
    let decoded: CaseStudyContent = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(decoded.meta.title, content.meta.title);
    assert_eq!(decoded.technologies, vec!["tech1"]);
}

#[test]
fn status_report_content_roundtrip() {
    let content = StatusReportContent {
        project: ProjectInfo {
            name: "Project".into(),
            client: "Client".into(),
            period_start: "2026-01-01".into(),
            period_end: "2026-01-15".into(),
            overall_rag: "green".into(),
            phase: "Execution".into(),
        },
        summary: "Good progress.".into(),
        milestones: vec![],
        budget: None,
        risks: vec![],
        actions: vec![],
        next_period_focus: "Finish implementation".into(),
        publishing: Default::default(),
    };

    let yaml = serde_yaml::to_string(&content).unwrap();
    let decoded: StatusReportContent = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(decoded.project.name, content.project.name);
}

#[test]
fn content_defaults_applied() {
    let yaml = r#"
engagement:
  title: "Engagement"
  date: "2026-01-01"
client:
  name: "Client"
executive_summary:
  headline: "Headline"
"#;
    let content: ProposalContent = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(content.engagement.version, "1.0");
    assert_eq!(content.engagement.language, "en");
    assert_eq!(content.engagement.subtitle, "");
}

// -- Document serialization tests --

#[test]
fn slide_document_roundtrip() {
    let doc = SlideDocument {
        resource_type: "SlideDocument@1".into(),
        slides: vec![
            Slide::Cover(CoverSlide {
                title: "Title".into(),
                subtitle: "Subtitle".into(),
                client: "Client".into(),
                reference: "REF".into(),
                date: "2026-01-01".into(),
                logo: "logo.png".into(),
            }),
        ],
    };

    let yaml = serde_yaml::to_string(&doc).unwrap();
    let decoded: SlideDocument = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(decoded.resource_type, "SlideDocument@1");
}

#[test]
fn report_document_roundtrip() {
    let doc = ReportDocument {
        resource_type: "ReportDocument@1".into(),
        meta: None,
        cover: None,
        chapters: vec![Chapter {
            title: "Intro".into(),
            sections: vec![],
        }],
    };

    let yaml = serde_yaml::to_string(&doc).unwrap();
    let decoded: ReportDocument = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(decoded.chapters[0].title, "Intro");
}

// -- ContentType tests --

#[test]
fn content_type_labels() {
    assert_eq!(ContentType::Proposal.label(), "ProposalContent");
    assert_eq!(ContentType::Brief.label(), "BriefContent");
    assert_eq!(ContentType::CaseStudy.label(), "CaseStudyContent");
    assert_eq!(ContentType::StatusReport.label(), "StatusReportContent");
}

#[test]
fn content_type_schema_is_valid_json() {
    for ct in [
        ContentType::Proposal,
        ContentType::Brief,
        ContentType::CaseStudy,
        ContentType::StatusReport,
    ] {
        let schema = ct.json_schema_str();
        let parsed: serde_json::Value = serde_json::from_str(&schema)
            .unwrap_or_else(|_| panic!("{:?} schema should be valid JSON", ct));
        assert!(parsed.as_object().is_some(), "{:?} schema should be an object", ct);
    }
}
