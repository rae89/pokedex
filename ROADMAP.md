# Roadmap

This roadmap outlines the planned direction for **pokemon-tui**. It is a living document — community input is welcome. If you'd like to contribute to any of these milestones, check the [GitHub Milestones](https://github.com/rae89/pokedex/milestones) for tracking and open an issue to discuss your approach before starting work.

---

## v0.2 — Polish Core

Refine the existing experience before adding new features.

- Improve navigation consistency and keyboard hints across all screens
- Better loading states and error handling UX
- Accessibility improvements (screen reader friendliness, color contrast)
- Performance: lazy loading, smarter caching

## v0.3 — Pokémon Data

Enrich the detail screen with deeper Pokémon data from the API.

- Evolution chains on the detail screen
- Abilities with full descriptions
- Move lists with sorting and filtering
- Egg groups, flavor text, and habitat info
- Richer detail screen layout

## v0.4 — Cross-Platform & Distribution

Make the TUI accessible on all major platforms.

- Linux builds in the CI release workflow
- Windows builds in the CI release workflow
- Publish to crates.io
- Homebrew formula
- Installation docs for all platforms

## v0.5 — Community & Extensibility

Lower the barrier for contributors and add extensibility.

- CONTRIBUTING.md with development guidelines
- GitHub issue templates (bug report, feature request)
- Theming and configuration file support
- Plugin or extension points

## v0.6 — Stat Analytics

Add statistical analysis and visualization tools for Pokémon data.

- Statistical distributions: visualize stat spreads across generations, types, or tiers
- Pokémon comparisons: side-by-side stat comparison tool
- Rankings and tier lists generated from base stat totals and type coverage scores
- Type coverage analyzer with deeper analysis beyond current team builder
- Charts and graphs rendered in TUI (bar charts, histograms)
