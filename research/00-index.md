# CLAN Research Notes — Index

> These notes document a full end-to-end independent QA, UX, and product audit of the CLAN (Context and Live Agent Notation) system, conducted on 2026-06-01. They may serve as primary source material for a paper on structured multi-agent context passing formats.

## File Index

| File | Contents |
|------|----------|
| [01-clan-system-overview.md](01-clan-system-overview.md) | What CLAN is, the file format, architecture, key concepts |
| [02-cli-full-test-results.md](02-cli-full-test-results.md) | Every CLI subcommand tested, exact commands, outputs, edge cases |
| [03-app-test-results.md](03-app-test-results.md) | Desktop app (Tauri + React) live test — launch, rendering, edit mode, debug log evidence |
| [04-without-clan-comparison.md](04-without-clan-comparison.md) | Side-by-side: how the same multi-agent pipeline works without CLAN vs with it |
| [05-workflow-impact-metrics.md](05-workflow-impact-metrics.md) | Quantitative measurements: token costs, file sizes, compression ratios, timing |
| [06-irish-adtech-market-analysis.md](06-irish-adtech-market-analysis.md) | The actual market analysis produced by the simulation — Irish ad tech agency OS market fit |
| [07-multi-agent-simulation.md](07-multi-agent-simulation.md) | Full details of both simulations (3-stage pipeline + 6-agent fan-out) |
| [08-bugs-and-issues.md](08-bugs-and-issues.md) | Every confirmed bug with file locations, reproduction steps, impact |
| [09-optimization-recommendations.md](09-optimization-recommendations.md) | All optimisation opportunities, priority-ranked with effort estimates |
| [10-good-bad-complex.md](10-good-bad-complex.md) | What CLAN does well, what it does badly, what is over-complex, final verdict |
| [11-new-patch-commands-update.md](11-new-patch-commands-update.md) | Test results for the 6 new surgical patch commands added in the latest update |

## Test Setup

- **CLAN version**: 1.0.0
- **Test workspace**: `/tmp/adtech-ie-sim/` (first simulation), `/tmp/adtech-ie-deep/` (deep simulation)
- **Constraint**: CLI tested as a black box from a clean directory — no access to CLAN repo files during test execution
- **Scenario**: Irish AdTech Agency OS — market fit analysis for an AI-powered operating system for Irish advertising agencies
- **Agents involved**: Market Researcher, Risk Analyst, Lead Partner (3-stage); Financial Analyst, Competitive Intel, Customer Discovery, Regulatory/Product, Synthesis Lead (6-agent fan-out)

## Key Findings at a Glance

**Verdict: CLAN is a sound system with a solid core concept. The CLI is near production-ready. The desktop app needs two targeted fixes before shipping.**

- 8 bugs confirmed (1 critical, 3 high, 4 medium/low)
- 15 optimisation opportunities identified
- ~65–75% token reduction vs raw JSON pipeline confirmed by measurement
- Cross-agent data sharing works correctly end-to-end
- One new bug class discovered during simulation: silent structured data loss when frontmatter omits `structured:` wrapper
- Double-save bug confirmed from live debug log evidence
