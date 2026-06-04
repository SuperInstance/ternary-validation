//! Validate ternary strategies against constraints.
//!
//! Provides `ValidationRule`, `StrategyValidator`, and `ValidationReport`
//! for checking bounds, conservation, and cascade patterns in ternary strategies.

/// A ternary trit value: -1, 0, or +1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

impl Trit {
    pub fn value(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Neg),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Pos),
            _ => None,
        }
    }
}

/// A ternary strategy is a sequence of trits.
#[derive(Debug, Clone)]
pub struct TernaryStrategy {
    pub positions: Vec<Trit>,
    pub label: String,
}

impl TernaryStrategy {
    pub fn new(label: &str, positions: Vec<Trit>) -> Self {
        Self { positions: positions.to_vec(), label: label.to_string() }
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// Sum of all trit values.
    pub fn weighted_sum(&self) -> i64 {
        self.positions.iter().map(|t| t.value() as i64).sum()
    }
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// A single validation finding.
#[derive(Debug, Clone)]
pub struct ValidationFinding {
    pub rule: String,
    pub message: String,
    pub severity: Severity,
    pub index: Option<usize>,
}

/// Result of validating a strategy.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub strategy_label: String,
    pub findings: Vec<ValidationFinding>,
    pub passed: bool,
}

impl ValidationReport {
    pub fn new(label: &str) -> Self {
        Self { strategy_label: label.to_string(), findings: Vec::new(), passed: true }
    }

    pub fn add(&mut self, finding: ValidationFinding) {
        if finding.severity == Severity::Error {
            self.passed = false;
        }
        self.findings.push(finding);
    }

    pub fn errors(&self) -> Vec<&ValidationFinding> {
        self.findings.iter().filter(|f| f.severity == Severity::Error).collect()
    }

    pub fn warnings(&self) -> Vec<&ValidationFinding> {
        self.findings.iter().filter(|f| f.severity == Severity::Warning).collect()
    }

    pub fn summary(&self) -> String {
        let e = self.errors().len();
        let w = self.warnings().len();
        let i = self.findings.iter().filter(|f| f.severity == Severity::Info).count();
        format!(
            "Strategy '{}' — {} errors, {} warnings, {} info — {}",
            self.strategy_label, e, w, i, if self.passed { "PASSED" } else { "FAILED" }
        )
    }
}

/// A rule that checks a constraint on a strategy.
pub trait ValidationRule {
    fn name(&self) -> &str;
    fn check(&self, strategy: &TernaryStrategy, report: &mut ValidationReport);
}

// --- Built-in rules ---

/// Ensures all positions are within bounds [0, max_len).
pub struct BoundsRule {
    pub max_len: usize,
}

impl BoundsRule {
    pub fn new(max_len: usize) -> Self {
        Self { max_len }
    }
}

impl ValidationRule for BoundsRule {
    fn name(&self) -> &str { "bounds" }
    fn check(&self, strategy: &TernaryStrategy, report: &mut ValidationReport) {
        if strategy.len() > self.max_len {
            report.add(ValidationFinding {
                rule: self.name().to_string(),
                message: format!("strategy length {} exceeds max {}", strategy.len(), self.max_len),
                severity: Severity::Error,
                index: None,
            });
        }
    }
}

/// Ensures the weighted sum equals a target (conservation).
pub struct ConservationRule {
    pub target_sum: i64,
}

impl ConservationRule {
    pub fn new(target: i64) -> Self { Self { target_sum: target } }
}

impl ValidationRule for ConservationRule {
    fn name(&self) -> &str { "conservation" }
    fn check(&self, strategy: &TernaryStrategy, report: &mut ValidationReport) {
        let sum = strategy.weighted_sum();
        if sum != self.target_sum {
            report.add(ValidationFinding {
                rule: self.name().to_string(),
                message: format!("weighted sum {} != target {}", sum, self.target_sum),
                severity: Severity::Error,
                index: None,
            });
        }
    }
}

/// Detects cascade patterns: too many consecutive same-sign trits.
pub struct CascadeRule {
    pub max_consecutive: usize,
}

impl CascadeRule {
    pub fn new(max: usize) -> Self { Self { max_consecutive: max } }
}

impl ValidationRule for CascadeRule {
    fn name(&self) -> &str { "cascade" }
    fn check(&self, strategy: &TernaryStrategy, report: &mut ValidationReport) {
        if strategy.positions.is_empty() {
            return;
        }
        let mut run = 1usize;
        for i in 1..strategy.positions.len() {
            if strategy.positions[i] == strategy.positions[i - 1] && strategy.positions[i] != Trit::Zero {
                run += 1;
                if run > self.max_consecutive {
                    report.add(ValidationFinding {
                        rule: self.name().to_string(),
                        message: format!("cascade of {} consecutive {:?} at index {}", run, strategy.positions[i], i),
                        severity: Severity::Warning,
                        index: Some(i),
                    });
                }
            } else {
                run = 1;
            }
        }
    }
}

/// Ensures no position is empty (strategy must have content).
pub struct NonEmptyRule;

impl ValidationRule for NonEmptyRule {
    fn name(&self) -> &str { "non_empty" }
    fn check(&self, strategy: &TernaryStrategy, report: &mut ValidationReport) {
        if strategy.is_empty() {
            report.add(ValidationFinding {
                rule: self.name().to_string(),
                message: "strategy is empty".to_string(),
                severity: Severity::Error,
                index: None,
            });
        }
    }
}

/// Validates that all trits are valid (always true for enum, but useful for raw data).
pub struct TritValidityRule;

impl ValidationRule for TritValidityRule {
    fn name(&self) -> &str { "trit_validity" }
    fn check(&self, _strategy: &TernaryStrategy, _report: &mut ValidationReport) {
        // Trit enum is always valid by construction; this is a no-op placeholder
        // for when strategies are built from raw data.
    }
}

/// Ensures minimum diversity: at least `min_unique` distinct trit values used.
pub struct DiversityRule {
    pub min_unique: usize,
}

impl DiversityRule {
    pub fn new(min: usize) -> Self { Self { min_unique: min } }
}

impl ValidationRule for DiversityRule {
    fn name(&self) -> &str { "diversity" }
    fn check(&self, strategy: &TernaryStrategy, report: &mut ValidationReport) {
        let has_neg = strategy.positions.iter().any(|t| *t == Trit::Neg);
        let has_zero = strategy.positions.iter().any(|t| *t == Trit::Zero);
        let has_pos = strategy.positions.iter().any(|t| *t == Trit::Pos);
        let unique = [has_neg, has_zero, has_pos].iter().filter(|&&b| b).count();
        if unique < self.min_unique {
            report.add(ValidationFinding {
                rule: self.name().to_string(),
                message: format!("only {} distinct trit values, need at least {}", unique, self.min_unique),
                severity: Severity::Warning,
                index: None,
            });
        }
    }
}

/// Runs a collection of rules against a strategy.
pub struct StrategyValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl StrategyValidator {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn validate(&self, strategy: &TernaryStrategy) -> ValidationReport {
        let mut report = ValidationReport::new(&strategy.label);
        for rule in &self.rules {
            rule.check(strategy, &mut report);
        }
        report
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for StrategyValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos_strategy() -> TernaryStrategy {
        TernaryStrategy::new("test", vec![Trit::Pos, Trit::Zero, Trit::Neg, Trit::Pos])
    }

    #[test]
    fn test_trit_values() {
        assert_eq!(Trit::Neg.value(), -1);
        assert_eq!(Trit::Zero.value(), 0);
        assert_eq!(Trit::Pos.value(), 1);
    }

    #[test]
    fn test_trit_from_i8() {
        assert_eq!(Trit::from_i8(-1), Some(Trit::Neg));
        assert_eq!(Trit::from_i8(0), Some(Trit::Zero));
        assert_eq!(Trit::from_i8(1), Some(Trit::Pos));
        assert_eq!(Trit::from_i8(2), None);
    }

    #[test]
    fn test_strategy_weighted_sum() {
        let s = pos_strategy();
        assert_eq!(s.weighted_sum(), 1);
    }

    #[test]
    fn test_empty_strategy_sum() {
        let s = TernaryStrategy::new("empty", vec![]);
        assert_eq!(s.weighted_sum(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_bounds_rule_pass() {
        let rule = BoundsRule::new(10);
        let s = pos_strategy();
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(report.passed);
    }

    #[test]
    fn test_bounds_rule_fail() {
        let rule = BoundsRule::new(2);
        let s = pos_strategy();
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(!report.passed);
        assert_eq!(report.errors().len(), 1);
    }

    #[test]
    fn test_conservation_rule_pass() {
        let rule = ConservationRule::new(1);
        let s = pos_strategy();
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(report.passed);
    }

    #[test]
    fn test_conservation_rule_fail() {
        let rule = ConservationRule::new(0);
        let s = pos_strategy();
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(!report.passed);
    }

    #[test]
    fn test_cascade_rule_no_cascade() {
        let rule = CascadeRule::new(2);
        let s = TernaryStrategy::new("no-cascade", vec![Trit::Pos, Trit::Neg, Trit::Pos]);
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(report.warnings().is_empty());
    }

    #[test]
    fn test_cascade_rule_detects() {
        let rule = CascadeRule::new(2);
        let s = TernaryStrategy::new("cascade", vec![Trit::Pos, Trit::Pos, Trit::Pos]);
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(!report.warnings().is_empty());
    }

    #[test]
    fn test_non_empty_rule_pass() {
        let rule = NonEmptyRule;
        let s = pos_strategy();
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(report.passed);
    }

    #[test]
    fn test_non_empty_rule_fail() {
        let rule = NonEmptyRule;
        let s = TernaryStrategy::new("empty", vec![]);
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(!report.passed);
    }

    #[test]
    fn test_diversity_rule_pass() {
        let rule = DiversityRule::new(3);
        let s = pos_strategy(); // has all three
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(report.warnings().is_empty());
    }

    #[test]
    fn test_diversity_rule_fail() {
        let rule = DiversityRule::new(3);
        let s = TernaryStrategy::new("uniform", vec![Trit::Pos, Trit::Pos]);
        let mut report = ValidationReport::new("test");
        rule.check(&s, &mut report);
        assert!(!report.warnings().is_empty());
    }

    #[test]
    fn test_validator_multiple_rules() {
        let v = StrategyValidator::new()
            .add_rule(Box::new(BoundsRule::new(10)))
            .add_rule(Box::new(ConservationRule::new(1)))
            .add_rule(Box::new(CascadeRule::new(3)));
        let report = v.validate(&pos_strategy());
        assert!(report.passed);
        assert_eq!(v.rule_count(), 3);
    }

    #[test]
    fn test_validator_fails_on_any_error() {
        let v = StrategyValidator::new()
            .add_rule(Box::new(BoundsRule::new(2)))
            .add_rule(Box::new(ConservationRule::new(1)));
        let report = v.validate(&pos_strategy());
        assert!(!report.passed);
    }

    #[test]
    fn test_report_summary() {
        let v = StrategyValidator::new()
            .add_rule(Box::new(NonEmptyRule));
        let report = v.validate(&pos_strategy());
        let summary = report.summary();
        assert!(summary.contains("PASSED"));
        assert!(summary.contains("test"));
    }

    #[test]
    fn test_report_summary_failed() {
        let v = StrategyValidator::new()
            .add_rule(Box::new(NonEmptyRule));
        let s = TernaryStrategy::new("empty", vec![]);
        let report = v.validate(&s);
        let summary = report.summary();
        assert!(summary.contains("FAILED"));
    }

    #[test]
    fn test_strategy_len() {
        let s = pos_strategy();
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn test_default_validator() {
        let v = StrategyValidator::default();
        assert_eq!(v.rule_count(), 0);
        let s = pos_strategy();
        let report = v.validate(&s);
        assert!(report.passed);
    }
}
