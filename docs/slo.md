# Service level promise to the Academic Council

Status: proposed for this semester (2026-11-20). Owner: Team 1.

The Rector asked for "100.00% uptime forever at 0.00 BGN". No system can promise that, and the university's lawyers warned that a written promise turns every second of delay into a penalty. This is what we can measure, what we promise, and what we do when we fall short.

## What we measure (indicators)

All indicators are measured at the production load balancer over a calendar month. Requests from our own health checks are not counted.

| Indicator | Good event | Counted events |
|---|---|---|
| Availability | A response that is not a 5xx and arrives within 5 seconds | Every request to sign-in, profile, catalogue and enrollment pages, and to `/oauth2/*` |
| Speed | The profile or catalogue page is served in under 1 second | Every profile and catalogue page request |
| Sign-in success | Sign-in completes, not counting a wrong password | Every sign-in attempt |
| Enrollment success | The request is recorded, or refused because the offering is full or closed | Every enrollment request |

## What we promise (objectives)

| Objective | Target per month | What it allows |
|---|---|---|
| Availability | 99.5% | About 3 hours 36 minutes of downtime in a 30-day month (0.5% of 43,200 minutes is 216 minutes) |
| Speed | 95% of pages under 1 second | |
| Sign-in success | 99.5% | |
| Enrollment success | 99.5% | |

Why not more: production runs on free and student-credit plans. One Azure region, one small database server, and no paid support. 99.5% is honest for that. Each extra "9" would cost money we do not have.

Staging (Render free plan) carries no promise. It sleeps after 15 minutes and its database is replaced every 30 days.

## The error budget and what happens when it runs out

The error budget is the 0.5% of failures each objective allows.

- **Budget left:** normal work. Deploys go out after review on staging.
- **Half the budget used before mid-month:** the team looks at the cause at the next meeting. Risky changes need a second reviewer.
- **Budget used up:** a change freeze. Only fixes for reliability and security are deployed until the next month starts. No load tests against production.
- **Enrollment day:** no deploys and no load tests in the 24 hours before enrollment opens, whatever the budget.

## How we keep it

- At least 2 replicas at all times, up to 10 under load. New replicas start before old ones stop.
- Enrollment counts places in one database statement, so a rush cannot oversell.
- The spike test (10 to 500 users in 30 seconds, held for 2 minutes) must pass: at least 99.5% success and 95% under 1 second.

## What is not covered

- Outages of Azure itself, and the free plan limits of the cloud providers.
- Modules built by other teams.
- Planned maintenance announced at least 48 hours ahead, outside 07:00 to 20:00 on working days.
