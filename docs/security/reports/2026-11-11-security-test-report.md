# Доклад от тестове за сигурност — 2026-11-11

## Обхват

- **Commit:** `57bbc76` (main). Базова линия: няма, това е първият преглед.
- **Режим:** първоначален пълен преглед: `security-audit` (Cloudflare), профил `quick`, след което `sec-threat-model`, `sec-requirements`, `sec-access-control`, `sec-monitoring` и `sec-devsecops`. `sec-auth-review` няма какво да прегледа, защото кодът за вход още не съществува; изискванията му са в SR-01…SR-09.
- **Метод:** статичен преглед. Кодът не е изпълняван и към продукция, Azure или GHCR не е изпращан трафик.
- **Скенери в CI:** няма резултати, защото `security.yml` се добавя с този клон.
- **Покритие:** частично. 8 от 15 единици са покрити, 4 са стигнали до кандидати, 3 са отложени.

## Резюме

- Потвърдени уязвимости: **0**
- За проверка от DevOps: **3** (V-01, V-02, V-03)
- Hardening бележки: **13** (1 оправена: H-11)
- Изисквания: 37, от тях 7 `met`, 2 `partially met`, 6 `not met` (2 от тях чакат и проверка от DevOps), 2 `needs validation`, 20 `n/a yet` (функциите още не съществуват)

Трите най-важни риска:

1. **V-01:** продукционната admin връзка към базата и `GHCR_PAT` вероятно са достъпни за workflow от всеки клон, преди code review.
2. **V-02:** деплоят по променлив tag позволява подмяна на продукционния образ от всеки с права за запис в пакета.
3. **V-03:** MySQL е достъпен от цял Azure, а приложението влиза като admin. Високият риск идва в комбинация с V-01.

## Нови констатации

Вж. `docs/security/VULNERABILITY-ASSESSMENT.md` (V-01 до V-03, H-01 до H-13).

## За проверка от DevOps

1. Къде са дефинирани `DATABASE_URL` и `GHCR_PAT`: repository, organization или environment? Има ли environment `production` с ограничение до `main` и reviewers? Кой има Write?
2. Кой има Write към GHCR пакета? Какви са scopes на `GHCR_PAT`? Container Apps ревизията по digest ли е закрепена?
3. Firewall правилата и `publicNetworkAccess` на MySQL. С кой потребител работи секретът `db-url`?
4. Какъв е federated subject на `unicloud-github-deployer` (H-02)?
5. Branch protection/rulesets на `main`, secret scanning и push protection, Dependabot alerts.

## Приложени мерки

- Добавен е `.github/workflows/security.yml`: CodeQL (security-extended), Trivy (зависимости, misconfiguration, credentials) и dependency review при PR. Всички actions са закрепени към commit SHA. Работи при PR, при push към `main` и всеки понеделник.
- Добавени са `AGENTS.md`, агентът `security-engineer` и skills `sec-*` за периодични прегледи.
- Изготвени са `THREAT-MODEL.md` (29 заплахи, T-01…T-29), `SECURITY-REQUIREMENTS.md` (37 изисквания, SR-01…SR-37, с опис на личните данни), `RBAC-MATRIX.md` (20 реда, R01…R20, с 4 решения за екипа) и `MONITORING-AND-IR.md` (12 събития за логване, 8 заявки за откриване, план за реакция при инцидент с уведомяване на КЗЛД до 72 часа).
- Добавени са `.github/dependabot.yml` (Maven, Actions, Docker, всеки понеделник), PR шаблон с чеклист за сигурност и изключения в `.gitignore` за локални credentials (H-11).

## Следващи стъпки

| Какво | Отговорник | Срок |
| --- | --- | --- |
| Отговори на въпроси 1–5 | DevOps | до края на седмица 1 |
| Environment `production` + преместване на тайните (V-01) | DevOps + Security | преди първия реален деплой с потребителски данни |
| Деплой по digest; `packages: write` само за main (V-02, H-06) | DevOps | седмица 2 |
| Отделен DB потребител за приложението, `VERIFY_IDENTITY` (V-03) | DevOps + Backend | седмица 2 |
| Решения по RBAC (R09, R11, R13, R15) и по отворените въпроси в Threat Model (притежание на факултетен номер, сесии или JWT, автентикация на услугите) | целият екип | преди PR-а с регистрация и вход |
| Включване на branch protection със задължителни `codeql`, `trivy`, `dependency-review`; secret scanning с push protection | собственикът на хранилището | след сливането на този PR |
| Пълен `sec-sweep` след като влязат регистрация, вход и записване | Security | при първия PR с тях |
