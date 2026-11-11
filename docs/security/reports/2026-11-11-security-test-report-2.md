# Доклад от тестове за сигурност — 2026-11-11 (2)

## Обхват

- **Клон:** `perf/smaller-docker-image`, базиран на `57bbc76` (main). Промяната още не е commit-ната; прегледан е работният `Dockerfile`.
- **Базова линия:** `git merge-base HEAD origin/main` = `57bbc76`.
- **Режим:** `sec-sweep --scope diff`. Променен е само `Dockerfile`.
- **Изпълнени skills:** `sec-vuln-assessment` (misconfiguration, контейнерен образ) и `sec-devsecops` (Docker build, supply chain). Останалите skills нямат тригер в тази промяна.
- **Метод:** статичен преглед и локални проверки на построения образ срещу локален Docker MySQL 8.4. Към продукция, Azure или GHCR не е изпращан трафик.
- **Скенери в CI:** `security.yml` още не е в `main`. Последното изпълнение (PR на `security/week1-security-baseline`, run `43104709438`): CodeQL и Trivy минават. `dependency-review` пада, защото Dependency graph не е включен в хранилището (вж. „За проверка от DevOps“). Отворени code-scanning alerts: 0. Trivy не е инсталиран локално и не е стартиран върху новия образ.

## Какво променя `Dockerfile`

- Runtime образът вече е `alpine:3.23` с Java runtime, построен с `jlink` само от нужните модули (определени с `jdeps`), вместо `eclipse-temurin:25-jre` (Ubuntu).
- Spring Boot jar-ът е разделен на слоеве (dependencies, loader, snapshot-dependencies, application).
- Размер на `linux/amd64` образа (както се push-ва в GHCR): компресиран 165 672 133 → 94 215 628 байта (−43.1%), некомпресиран 404 066 304 → 128 897 536 байта (−68.1%).

## Резюме

- Потвърдени уязвимости: **0**
- За проверка от DevOps: **0 нови** (V-01 до V-03 не са засегнати от промяната)
- Hardening бележки: **1 нова** (H-14), **1 обновена** (H-07)
- Регресия, открита и поправена в същия клон, преди деплой: процесът работеше с група `root` (вж. „Приложени мерки“).

Трите най-важни наблюдения:

1. **Повърхността за атака на образа намалява:** 16 Alpine пакета срещу 106 Debian пакета, без JDWP агент, без `jdk.attach` и без отдалечен JMX агент.
2. **H-14:** CI не сканира построения образ за CVE в OS пакетите (само `trivy fs`). Пропускът съществуваше и преди, но сега базата е различна и си струва да се затвори.
3. **H-07 (обновена):** новите базови образи също не са закрепени по digest.

## Нови и променени констатации

| ID | Статус | Severity | Заглавие | Къде |
| --- | --- | --- | --- | --- |
| H-07 | hardening (обновена) | — | Базовите образи `eclipse-temurin:25-jdk-alpine` и `alpine:3.23` не са закрепени по digest | `Dockerfile:3,25` |
| H-14 | hardening (нова) | — | Построеният образ не се сканира за уязвимости в OS пакетите и Java runtime-а | `.github/workflows/deploy.yml:65-66`; `.github/workflows/security.yml` (клон `security/week1-security-baseline`) |

**H-07.** Закрепването по digest без автоматично обновяване би замразило образа без security корекции на Alpine. Препоръка: digest заедно с `.github/dependabot.yml` за `docker` (седмично), както предлага `sec-devsecops`. Дотогава таговете `3.23` и `25-jdk-alpine` получават корекциите автоматично при всеки build.

**H-14.** `trivy fs` проверява `pom.xml` и `Dockerfile`, но не и пакетите в образа (`musl`, `busybox`, `libcrypto3`, Java runtime). Минимална мярка: в job-а `build` на `deploy.yml`, след `docker build`, стъпка `aquasecurity/trivy-action` с `scan-type: image`, `image-ref: ${{ steps.tag.outputs.image }}`, `severity: HIGH,CRITICAL`, `ignore-unfixed: true`, `exit-code: "1"`, закрепена към същия SHA като в `security.yml`. Така образът се проверява преди push към GHCR.

Тези два реда трябва да се пренесат в `docs/security/VULNERABILITY-ASSESSMENT.md`, когато `security/week1-security-baseline` и този клон се слеят (регистърът още не е в `main`).

## Проверки, които минаха

Проверено с построения образ `unicloud-portal:slim`, сравнено с предишния образ от `57bbc76`:

- **Непривилегирован потребител:** `uid=1000 gid=1000`, без допълнителни групи. Предишният образ работеше като `ubuntu` с групи `adm` и `sudo` (без `sudo` binary).
- **Права върху файловете:** `/app` и `/opt/java` са `root:root`, само за четене от процеса (`touch` → Permission denied). Записваеми за всички са само `/tmp` и `/var/tmp` (sticky bit), които Tomcat използва.
- **Без тайни в образа:** `ARG`/`ENV` съдържат само `GIT_SHA` и пътища. `COPY` копира изрично само `.mvn/`, `mvnw`, `pom.xml` и `src/`, така че `.env` или ключове не могат да попаднат в образа, дори ако `.dockerignore` не ги изключва (H-11).
- **Trust store:** същите 118 сертификата като в Temurin JRE, включително `DigiCert Global Root G2` и `Microsoft RSA Root Certificate Authority 2017`. Преминаването към `sslMode=VERIFY_IDENTITY` (мярка за V-03) ще работи без допълнителни сертификати.
- **TLS политика:** `jdk.tls.disabledAlgorithms` и останалите `java.security` свойства са идентични с тези на предишния образ. `crypto.policy=unlimited`.
- **Премахнати security providers:** само `SunPKCS11` (HSM), `SunPCSC` (смарт карти), `XMLDSig` и `JdkSASL`. Приложението не ги използва. Списъкът с модули се изчислява наново от `jdeps` при всеки build, така че нова зависимост, която ги изисква статично (напр. XML подпис при SAML SSO), ще ги добави автоматично.
- **Без инструменти за отдалечен достъп до JVM:** `-agentlib:jdwp` не се зарежда (няма `libjdwp.so`). Модулите `jdk.attach`, `jdk.management.agent` и `jdk.jdwp.agent` не са включени.
- **TLS към MySQL:** връзките на приложението към локален MySQL 8.4 с `sslMode=REQUIRED` използват TLSv1.3 (`performance_schema.status_by_thread`).
- **Работа:** Flyway прилага V1, Hibernate `validate` минава, `/health` връща подадения `GIT_SHA`, стартиране за 2.3 s.
- **Graceful shutdown:** `java` е PID 1 и получава SIGTERM. Hikari и JPA се затварят нормално.
- **Деплой проверката покрива DNS и връзката към базата:** приложението не стартира без успешен Flyway, така че разлика в DNS резолвера на musl спрямо glibc би се видяла в стъпката „Check the new version is live“, преди трафикът да се пренасочи.
- **Тагове:** без `:latest`. Няма `apk add`, така че няма пакети извън базовия образ.

## За проверка от DevOps

1. Включен ли е **Dependency graph** (Settings → Code security)? Без него `dependency-review` в `security.yml` пада при всеки PR.

## Приложени мерки

- **`USER 1000:1000` вместо `USER 1000`** (работен `Dockerfile`, още не е commit-нат). В Alpine няма запис за uid 1000, затова без изрична група процесът получаваше `gid=0` (root). В базовия образ `/etc/network/if-up.d/dad` е `root:root 0775`, т.е. записваем за група root. Няма достижим ефект (файлът не се изпълнява в контейнера), но правото е излишно. Регресията е въведена и поправена в този клон и не е стигала до продукция.

## Следващи стъпки

| Какво | Отговорник | Срок |
| --- | --- | --- |
| Trivy image scan в `build` job-а преди push (H-14) | DevOps + Security | заедно със сливането на `security.yml` |
| `dependabot.yml` за `docker`, `maven`, `github-actions`, после digest pinning (H-07) | DevOps | седмица 2 |
| Включване на Dependency graph | Repo admin | седмица 1 |
| Пренасяне на H-07 и H-14 в регистъра след сливане на двата клона | Security | при сливане |
