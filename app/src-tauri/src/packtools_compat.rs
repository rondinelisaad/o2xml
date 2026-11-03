use serde::Deserialize;
use xml_schematron::SvrlReport;

const AUTHOR_NOTES_RULES: &str =
    include_str!("../resources/sps/packtools-4.18.3/validation_rules/author_notes_rules.json");
const FN_RULES: &str =
    include_str!("../resources/sps/packtools-4.18.3/validation_rules/fn_rules.json");
const ARTICLE_TYPE_RULES: &str =
    include_str!("../resources/sps/packtools-4.18.3/validation_rules/article_type_rules.json");

#[derive(Deserialize)]
struct AuthorNotesRuleFile {
    author_notes_rules: FnTypeRules,
}

#[derive(Deserialize)]
struct FnRuleFile {
    fn_rules: FnTypeRules,
}

#[derive(Deserialize)]
struct FnTypeRules {
    fn_type_expected_values: Vec<String>,
}

#[derive(Deserialize)]
struct ArticleTypeRuleFile {
    article_type_rules: ArticleTypeRules,
}

#[derive(Deserialize)]
struct ArticleTypeRules {
    sub_article_type_list: Vec<String>,
}

/// Reconciles findings emitted by the legacy Packtools Schematron with the
/// authoritative value lists used by Packtools 4.18.3's Python validator.
///
/// Packtools still ships its older Schematron unchanged, so running that file
/// alone reports values which the current validation layer explicitly accepts.
/// This overlay is intentionally narrow and fail-closed: malformed or missing
/// pinned rule files preserve every Schematron finding.
pub(super) fn reconcile_sps_1_10(report: &mut SvrlReport) -> Result<usize, serde_json::Error> {
    let author_notes: AuthorNotesRuleFile = serde_json::from_str(AUTHOR_NOTES_RULES)?;
    let fn_rules: FnRuleFile = serde_json::from_str(FN_RULES)?;
    let article_types: ArticleTypeRuleFile = serde_json::from_str(ARTICLE_TYPE_RULES)?;

    let accepts_coi_statement = author_notes
        .author_notes_rules
        .fn_type_expected_values
        .iter()
        .any(|value| value == "coi-statement");
    let accepts_data_availability = fn_rules
        .fn_rules
        .fn_type_expected_values
        .iter()
        .any(|value| value == "data-availability");
    let accepts_reviewer_report = article_types
        .article_type_rules
        .sub_article_type_list
        .iter()
        .any(|value| value == "reviewer-report");

    let before = report.findings.len();
    report.findings.retain(|finding| {
        let location = finding.location.as_deref();
        let obsolete = (accepts_coi_statement
            && location == Some("/article/front/article-meta/author-notes/fn")
            && finding.message
                == "Element 'fn', attribute fn-type: Invalid value 'coi-statement'.")
            || (accepts_data_availability
                && location == Some("/article/back/fn-group/fn")
                && finding.message
                    == "Element 'fn', attribute fn-type: Invalid value 'data-availability'.")
            || (accepts_reviewer_report
                && location == Some("/article/sub-article")
                && finding.message
                    == "Element 'sub-article', attribute article-type: Invalid value 'reviewer-report'.");
        !obsolete
    });
    Ok(before.saturating_sub(report.findings.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xml_domain::Severity;
    use xml_schematron::{FindingKind, SvrlFinding};

    fn finding(location: &str, message: &str) -> SvrlFinding {
        SvrlFinding {
            kind: FindingKind::FailedAssert,
            severity: Severity::Error,
            message: message.to_owned(),
            location: Some(location.to_owned()),
            test: None,
            id: None,
            role: None,
            flag: None,
        }
    }

    #[test]
    fn suppresses_only_legacy_findings_accepted_by_packtools_4_18_3(
    ) -> Result<(), serde_json::Error> {
        let mut report = SvrlReport {
            findings: vec![
                finding(
                    "/article/front/article-meta/author-notes/fn",
                    "Element 'fn', attribute fn-type: Invalid value 'coi-statement'.",
                ),
                finding(
                    "/article/back/fn-group/fn",
                    "Element 'fn', attribute fn-type: Invalid value 'data-availability'.",
                ),
                finding(
                    "/article/sub-article",
                    "Element 'sub-article', attribute article-type: Invalid value 'reviewer-report'.",
                ),
                finding(
                    "/article/sub-article",
                    "Element 'sub-article': Missing attribute id.",
                ),
            ],
            truncated: false,
        };

        let removed = reconcile_sps_1_10(&mut report)?;

        assert_eq!(removed, 3);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].message,
            "Element 'sub-article': Missing attribute id."
        );
        Ok(())
    }

    #[test]
    fn preserves_same_value_outside_the_packtools_context() -> Result<(), serde_json::Error> {
        let mut report = SvrlReport {
            findings: vec![finding(
                "/article/body/fn",
                "Element 'fn', attribute fn-type: Invalid value 'coi-statement'.",
            )],
            truncated: false,
        };

        let removed = reconcile_sps_1_10(&mut report)?;

        assert_eq!(removed, 0);
        assert_eq!(report.findings.len(), 1);
        Ok(())
    }
}
