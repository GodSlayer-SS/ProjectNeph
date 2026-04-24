# Invariants (do not break casually)

1. **Local-first:** primary data in SQLite; no mandatory cloud.
2. **BYOK** for third-party LLM; keys in OS secret store, not in repo.
3. **Migrations** are the only way to evolve schema in released builds.
4. **File mutations** that can lose data are **red**-tier in the product intent; they must not silently bypass future backend confirmation.
5. **Changelog** records user-visible and trust-relevant changes; do not “quiet ship” security behavior changes.
6. **Alpha honesty:** the product is not marketed as “secure by default for strangers” until the audit gates are green.
