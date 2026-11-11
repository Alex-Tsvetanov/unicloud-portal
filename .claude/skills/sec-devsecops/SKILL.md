---
name: sec-devsecops
description: Keep UniCloud's automated security workflow and CI/CD supply chain sound - the security.yml scans (CodeQL, Trivy, dependency review), deploy.yml hardening (fork PRs, token permissions, action pinning, OIDC subject, image push), branch protection, Dependabot, and the secure-coding PR checklist developers follow. Use for "automate security", "CI security", "pipeline", "supply chain", "DevSecOps", "pin actions", "PR checklist", "train the team", or when .github/, Dockerfile, pom.xml or infra/ change.
---

# DevSecOps: automation, pipeline and developer guidance

Read `AGENTS.md` first.

## 1. Automated scans (`.github/workflows/security.yml`)

The workflow runs on every pull request, on pushes to `main`, and weekly. It should contain, with read-only `contents` and only the extra permissions each job needs:

- **CodeQL** (`java-kotlin`, `build-mode: none`, `security-extended` queries) → code-scanning alerts.
- **Trivy filesystem scan** (vulnerabilities in `pom.xml` dependencies, misconfiguration in `Dockerfile`/workflows, credentials) → SARIF to code scanning; fails only on HIGH/CRITICAL with a fix available.
- **Dependency review** on pull requests: blocks PRs that add a dependency with a HIGH/CRITICAL advisory.

When editing it: pin every third-party action to a full commit SHA with the version in a comment (look up with `gh api repos/<owner>/<repo>/commits/<tag> -q .sha`), keep `permissions:` per job, never add secrets to it, and never use `pull_request_target`. Ask the team to make the CodeQL and dependency-review checks required in branch protection (needs repo admin).

## 2. Review `deploy.yml` and the release path

Check and report (findings or hardening notes, per the evidence bar):

- Triggers: `pull_request` (not `pull_request_target`), so fork PRs get a read-only token and no secrets. Pushing and deploying happen only on `refs/heads/main`.
- `permissions:` per job; `packages: write` and `id-token: write` only where needed. Note when a job holds `packages: write` while running PR code (it is gated by `if: main` for the push step, but the token exists in the job).
- Actions pinned to SHAs (`actions/checkout@v4` style tags are mutable: hardening, or a finding if an action is from a small third party).
- No `${{ github.event.* }}` user-controlled values interpolated into `run:` scripts (script injection). Use `env:` indirection.
- Docker build: base images pinned by digest (hardening); `--build-arg` carries no secrets.
- Azure: OIDC federated subject `repo:<owner>/<repo>:ref:refs/heads/main`; Contributor scoped to the resource group (wider than needed: a Container Apps-only custom role is the least-privilege option). Registry credentials: `GHCR_PAT` long-lived PAT vs. making the package public or using a managed identity.
- `concurrency: production` present (it is) so two deploys never race.

## 3. Repository settings (needs validation by a repo admin)

Branch protection or rulesets on `main` (PR required, approvals, required checks `test`, `build`, `CodeQL`, `dependency-review`, dismiss stale reviews, no force push); secret scanning and push protection enabled; Dependabot alerts and security updates enabled; `CODEOWNERS` with the security engineer on `.github/`, `infra/`, `pom.xml`, `Dockerfile`, security config and migrations; Actions setting "Require approval for fork PR workflows".

Propose `.github/dependabot.yml` (maven, github-actions, docker; weekly) and `CODEOWNERS` as PRs when missing.

## 4. Secure-coding PR checklist (training)

Keep this list in the pull request template (`.github/pull_request_template.md`) once the team agrees; when reviewing a PR, walk it:

- [ ] New endpoint has an explicit access rule and a test that another student gets 403/404.
- [ ] IDs from the request are checked against the logged-in user (or come from the principal, e.g. `/me`).
- [ ] Request DTOs don't accept `role`, `enabled`, `status`, `id`.
- [ ] SQL uses `?`/named parameters only.
- [ ] No credentials, tokens or real personal data in code, tests or logs.
- [ ] Personal data in responses is limited to what the caller needs.
- [ ] New dependency is maintained and needed; no HIGH/CRITICAL advisories.
- [ ] Migration is backward compatible and adds no personal data without a stated purpose.

When asked to "train" or "explain", produce a short session note (Bulgarian) per topic with a vulnerable and a fixed snippet taken from this codebase's style (Spring Boot, `JdbcTemplate`), not generic examples.

## Output

Findings in the `sec-sweep` shape, plus a list of proposed config changes (files and diffs). Don't change branch protection or repository settings yourself.
