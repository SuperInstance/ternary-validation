# ternary-validation: Rule-based validation of ternary strategy sequences

Define constraints—bounds, conservation, cascade limits, diversity requirements—and run them against ternary strategies to produce pass/fail reports with severity-annotated findings.

## Why This Exists

A ternary strategy is only useful if it satisfies your constraints. Maybe it can't be too long, maybe the sum of signals must be zero (conservation), maybe you can't have five consecutive positive signals (cascade), maybe you need all three trit values present (diversity). This crate gives you composable validation rules that produce structured reports, so you can catch invalid strategies early and explain exactly what went wrong.

## Core Concepts

- **Trit** — A ternary value: `Neg` (−1), `Zero` (0), or `Pos` (+1).
- **TernaryStrategy** — A named sequence of trits with a label. Has a `weighted_sum()` (sum of all trit values as i64).
- **ValidationRule** — A trait. Each rule has a `name()` and a `check()` method that inspects a strategy and adds findings to a report. Rules don't return pass/fail directly—they annotate the report.
- **ValidationReport** — Accumulates findings. Tracks pass/fail status: any finding with `Error` severity causes the report to fail. Warnings and info don't affect the pass/fail outcome.
- **ValidationFinding** — A single issue: which rule found it, a human-readable message, severity (Info/Warning/Error), and optional position index.
- **Severity** — `Info` (noteworthy but fine), `Warning` (potential problem), `Error` (constraint violated).
- **StrategyValidator** — Holds a collection of `Box<dyn ValidationRule>` and runs them all against a strategy.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-validation = "0.1"
```

```rust
use ternary_validation::*;

let strategy = TernaryStrategy::new("my_strategy", vec![
    Trit::Pos, Trit::Zero, Trit::Neg, Trit::Pos,
]);

// Build a validator with multiple rules
let validator = StrategyValidator::new()
    .add_rule(Box::new(BoundsRule::new(100)))          // max 100 positions
    .add_rule(Box::new(ConservationRule::new(1)))       // weighted sum must equal 1
    .add_rule(Box::new(CascadeRule::new(3)))            // no more than 3 consecutive same-sign
    .add_rule(Box::new(NonEmptyRule))                   // must have content
    .add_rule(Box::new(DiversityRule::new(2)));         // at least 2 distinct trit values

let report = validator.validate(&strategy);
println!("{}", report.summary());
// Strategy 'my_strategy' — 0 errors, 0 warnings, 0 info — PASSED

// Inspect individual findings
if !report.passed {
    for error in report.errors() {
        println!("ERROR [{}]: {}", error.rule, error.message);
    }
    for warning in report.warnings() {
        println!("WARN  [{}]: {}", warning.rule, error.message);
    }
}
```

## API Overview

| Type | What it is |
|---|---|
| `Trit` | Ternary value: `Neg`, `Zero`, `Pos` |
| `TernaryStrategy` | Named trit sequence with weighted sum |
| `Severity` | `Info`, `Warning`, or `Error` |
| `ValidationFinding` | One issue: rule name, message, severity, optional index |
| `ValidationReport` | Accumulator for findings; tracks pass/fail |
| `ValidationRule` | Trait: `name()` + `check(strategy, report)` |
| `StrategyValidator` | Runs a collection of rules against a strategy |
| `BoundsRule` | Checks strategy length ≤ max |
| `ConservationRule` | Checks weighted sum == target |
| `CascadeRule` | Warns on runs of same-sign trits > limit |
| `NonEmptyRule` | Errors on empty strategies |
| `DiversityRule` | Warns if fewer than N distinct trit values |
| `TritValidityRule` | No-op placeholder for raw-data validation |

## How It Works

**Rule execution.** `StrategyValidator::validate` creates a fresh `ValidationReport`, then iterates all registered rules, calling `check(strategy, &mut report)` on each. Rules add findings directly to the report via `report.add(finding)`. The report's `passed` flag flips to `false` on the first `Error`-severity finding.

**Built-in rules.** `BoundsRule` compares `strategy.len()` against a maximum. `ConservationRule` compares `strategy.weighted_sum()` against a target. `CascadeRule` scans for runs of consecutive non-zero trits with the same sign, emitting a warning for each run exceeding the limit. `NonEmptyRule` checks that the strategy has at least one position. `DiversityRule` counts how many distinct trit values appear (Neg, Zero, Pos) and warns if below the minimum.

**Report structure.** `ValidationReport` stores all findings in order of discovery. `errors()` and `warnings()` return filtered views. `summary()` produces a one-line string: `Strategy 'label' — N errors, M warnings, K info — PASSED/FAILED`.

**Custom rules.** Implement `ValidationRule` for your own types. Each rule is self-contained: it receives the strategy and a mutable reference to the report. This makes rules composable—any combination works without interference.

## Known Limitations

- **TritValidityRule is a no-op.** Since `Trit` is an enum, it's always valid by construction. This rule exists as a placeholder for when strategies are built from raw integer data (which this crate doesn't currently support).
- **No cross-strategy validation.** Each rule operates on a single strategy in isolation. There's no built-in support for validating relationships between strategies (e.g., "strategy A's sum must equal strategy B's sum").
- **Cascade warnings are per-run, not per-strategy.** If a strategy has a run of 5 consecutive positive trits, `CascadeRule::new(3)` emits warnings at indices 3 and 4 (the 4th and 5th trit). Each warning points to the trit that exceeded the limit, not the start of the run.
- **No error recovery or partial validation.** All rules run regardless of whether previous rules found errors. There's no "stop on first error" mode or rule dependency ordering.

## Use Cases

- **Strategy admission control.** Before passing a strategy to the execution engine, validate it against structural constraints. Reject strategies that violate bounds or conservation laws.
- **Data quality checks.** Apply cascade and diversity rules to incoming ternary data streams. Flag sequences that look anomalous (all positive, no diversity, suspicious runs).
- **CI/CD for strategy generation.** Validate generated strategies in automated tests. `assert!(report.passed)` catches regressions in strategy construction logic.

## Ecosystem Context

Validates strategies produced by `ternary-grammar` (parsed and evaluated expressions), `ternary-search` (paths through strategy graphs), and `ternary-pipeline` (processed output). No direct dependencies on other ternary crates.

## License

MIT

## See Also
- **ternary-metrics** — related
- **ternary-fitness** — related
- **ternary-scoring** — related
- **ternary-experiment** — related
- **ternary-benchmark** — related

