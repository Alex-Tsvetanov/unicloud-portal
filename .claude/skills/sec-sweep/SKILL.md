---
name: sec-sweep
description: Periodic UniCloud security sweep that runs the Security Engineer skills in order and writes a dated Security Test Report. Use for "run the security sweep", "weekly security check", "security status", "security test report", before an enrollment campaign, or on a pull request that touches auth, roles, controllers, SQL, migrations, dependencies, Dockerfile, workflows or infra. Accepts --scope diff (only what changed since the base branch), --deep (adds a full security-audit run) and --since <sha>.
---

# Security sweep

The entry point for periodic security work on this repository. It decides what to review, runs the focused skills, and writes one dated **Security Test Report**. Read `AGENTS.md` first: its ground rules, evidence bar and disclosure rule apply throughout.

## 1. Decide the scope

1. Record `HEAD` (`git rev-parse HEAD`) and whether the worktree is dirty.
2. Find the baseline:
   - `--scope diff`: `git merge-base HEAD origin/main` (fall back to `upstream/main`).
   - `--since <sha>`: that SHA.
   - otherwise: the SHA in `docs/security/.last-sweep`; if missing, this is the first sweep and everything is in scope.
3. List changed files: `git diff --name-only <baseline>...HEAD` (on a first sweep, `git ls-files`), excluding `.claude/` and `docs/`.
4. Map changes to skills. A skill runs if any of its triggers changed, or if its deliverable does not exist yet:

| Changed paths or content | Skills |
| --- | --- |
| new or changed controllers, `@RequestMapping`/`@GetMapping`/..., DTOs, services | `sec-access-control`, `sec-vuln-assessment` |
| anything matching `Security`, `auth`, `login`, `register`, `password`, `token`, `session`, `jwt`, `oauth` | `sec-auth-review`, `sec-access-control` |
| `db/migration/*.sql`, entities, repositories, `JdbcTemplate`, `@Query` | `sec-vuln-assessment`, `sec-data-protection` (new personal-data columns) |
| `pom.xml`, `mvnw`, `.mvn/`, `Dockerfile` | `sec-vuln-assessment` (dependencies/image), `sec-devsecops` |
| `.github/**`, `infra/**`, `application*.properties` | `sec-devsecops`, `sec-data-protection`, `sec-threat-model` |
| new component, new external integration, new data flow, new team consuming the API | `sec-threat-model`, `sec-requirements` |
| logging, audit, error handling | `sec-monitoring` |

   A weekly full sweep always runs `sec-threat-model` in "check for drift" mode and `sec-requirements` in "verify status" mode, even when nothing in their trigger list changed.

## 2. Collect automated scanner results

If `gh` is authenticated and the repository has the `security` workflow, read the latest results instead of re-scanning:

```bash
gh run list --workflow security.yml --limit 3
gh api "repos/{owner}/{repo}/code-scanning/alerts?state=open&per_page=100" --jq '.[] | [.rule.security_severity_level // .rule.severity, .rule.id, .most_recent_instance.location.path, .most_recent_instance.location.start_line] | @tsv'
```

If they're unavailable, say so in the report and continue; the focused skills still do source review. Do not install scanners or download tools as part of a sweep.

## 3. Run the focused skills

Run each selected skill in this order, so later ones can reference earlier IDs:

1. `sec-threat-model`
2. `sec-requirements`
3. `sec-access-control`
4. `sec-auth-review`
5. `sec-vuln-assessment`
6. `sec-data-protection`
7. `sec-devsecops`
8. `sec-monitoring`

When sub-agents are available, run 3 to 8 in parallel (they write different sections) after 1 and 2 are done, giving each the scope list from step 1. Each returns findings in the shape below; the sweep, not the sub-skill, writes the report.

With `--deep`, also run the `security-audit` skill in full audit mode (`quick` profile for a weekly run, `standard` before an enrollment campaign), with its output directory outside the repo (its default `~/security-audit-skill/unicloud-portal/run-<N>`). Prior runs there are additive, so always use the same parent directory. Import its `confirmed` and `needs_validation` records into the vulnerability register (`V-xx`), keeping the audit's fingerprint in the record.

## 4. Finding shape

Every item any skill reports uses this shape so the register can deduplicate:

```text
id:        V-xx (new) or existing ID
status:    confirmed | needs-validation | hardening | fixed | accepted-risk | retired
severity:  critical | high | medium | low | info   (confirmed only)
title:     one line
where:     path:line (one or more)
actor:     who can do it (anonymous, any student, teacher, inspector, CI contributor, ...)
boundary:  which control should stop it and what is crossed
evidence:  the source trace, or for needs-validation the exact unresolved fact and who can answer it
fix:       smallest effective change + regression test idea
refs:      T-xx / SR-xx / RBAC row / OWASP / CWE
```

Before adding an item, search `docs/security/VULNERABILITY-ASSESSMENT.md` for the same root cause. Update the existing ID rather than creating a duplicate. Re-verify every open item whose files changed; mark it `fixed` only with a pointer to the fixing commit and the test that proves it.

## 5. Write the Security Test Report

Create `docs/security/reports/YYYY-MM-DD-security-test-report.md` (add `-2`, `-3` for more sweeps the same day) with:

1. **Обхват** — commit SHA, baseline, mode (diff / weekly / deep), skills run, scanner results used or missing.
2. **Резюме** — counts by status and severity; the three most important risks in one line each; trend vs. the previous report.
3. **Нови и променени констатации** — table of V-xx with status, severity, title, where.
4. **Проверки, които минаха** — what was checked and found sound (this is evidence too; list the checks, not just "OK").
5. **За проверка от DevOps** — every needs-validation item as a precise question.
6. **Приложени мерки** — fixes merged since the previous report, with commit SHAs.
7. **Следващи стъпки** — owner and target date per open high/medium item.

Respect the disclosure rule in `AGENTS.md`: a confirmed high/critical issue in deployed code appears only as "V-xx: under private remediation".

## 6. Close the sweep

- Update `docs/security/.last-sweep` with the reviewed SHA, only on a full (not `--scope diff`) sweep.
- Print a short summary: report path, counts, top risks, files changed.
- Do not commit or push unless asked. On a pull request, offer the report summary as a PR comment.
