# Changelog

## v1.0.0-rc1 - 2026-03-20

### Hardened
- audited and aligned public CLI command surface/help text for onboarding, authoring, and maintenance flows
- aligned dependency/lock/verify/repair messaging with actual behavior and profile semantics
- fixed verify/repair profile handling to evaluate lock state for the selected `--profile`
- refined dependency install messaging to remove stale deferred-language contradictions

### Documentation
- consolidated README around rc1 positioning, scope boundaries, and demoable workflows
- updated onboarding, authoring, registry, trust, dependency, lock, profiles, repair, and roadmap docs for rc1
- refreshed trust-policy wording to distinguish verified vs recorded vs deferred behaviors

### Tests and quality
- expanded integration coverage for profile-specific verify behavior and CLI help contract
- validated repository with `cargo fmt`, strict `cargo clippy`, and full `cargo test`

### Still intentionally post-1.0
- full SAT/global dependency conflict solving
- transactional rollback guarantees
- hosted publish services and remote transport/runtime distribution features
