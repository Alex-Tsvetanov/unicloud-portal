# Наблюдение на сигурността и реакция при инциденти

| | |
| --- | --- |
| Последна актуализация | 2026-11-11 |
| Преглеждан commit | `57bbc76` (main) |
| Автор | security-engineer агент; преглед: Алекс (Security Engineer) |
| Поддържа се от | skill `sec-monitoring` |

## 1. Събития, които приложението трябва да логва (SR-27)

Всяко събитие е **един ред JSON към stdout**: Container Apps ги изпраща към Log Analytics. Без пароли, токени и пълни профили; потребителят се идентифицира по `userId`. Входните данни се почистват от нови редове, за да няма log injection.

Общи полета: `ts` (UTC), `event`, `userId` (или `null`), `ip` (от `X-Forwarded-For`, с `server.forward-headers-strategy=framework`), `replica`, `outcome`.

| `event` | Допълнителни полета | Източник в Spring |
| --- | --- | --- |
| `auth.login.success` | `userId`, `role` | `AuthenticationSuccessEvent` |
| `auth.login.failure` | `identifierHash` (SHA-256 на имейла с pepper; не самият имейл), `reason` | `AbstractAuthenticationFailureEvent` |
| `auth.throttled` | `identifierHash`, `counter` | rate limiter |
| `auth.logout` | `userId` | logout handler |
| `account.registered` | `userId`, `role` (винаги STUDENT) | registration service |
| `account.role_changed` | `actorId`, `targetId`, `from`, `to` | admin service |
| `account.enabled_changed` | `actorId`, `targetId`, `from`, `to` | admin service |
| `student.status_changed` | `actorId`, `studentId`, `from`, `to` | inspector service |
| `access.denied` | `userId`, `method`, `pathTemplate` (напр. `/api/students/{id}`), `status` | `AuthorizationDeniedEvent` / `AccessDeniedHandler` |
| `enrollment.created` / `enrollment.cancelled` | `studentId`, `offeringId`, `actorId` (ако не е самият студент), `result`: `ok` / `full` / `closed` / `not_active` / `duplicate` | enrollment service |
| `offering.changed` | `actorId`, `offeringId`, промени в `capacity` и `is_open` | inspector service |
| `service_api.read` | `clientId`, `endpoint`, `records` | filter за API ключа |

Към `57bbc76` никое от тези събития не се логва: функциите още не съществуват.

## 2. Откриване: заявки и alerts (Azure Log Analytics)

Таблицата е `ContainerAppConsoleLogs_CL`; съобщението е в `Log_s`. Имената на колоните трябва да се потвърдят веднъж от DevOps. Alert rules в Azure Monitor с тези заявки са безплатният вариант на SIEM.

```kusto
// Общ парсер: запазете го като функция SecEvents
let SecEvents = ContainerAppConsoleLogs_CL
  | where ContainerAppName_s == "unicloud-portal"
  | extend e = parse_json(Log_s)
  | where isnotempty(e.event)
  | project TimeGenerated, event = tostring(e.event), userId = tostring(e.userId),
            ip = tostring(e.ip), id = tostring(e.identifierHash), e;
```

| # | Заплаха | Заявка | Праг | Реакция |
| --- | --- | --- | --- | --- |
| D1 | Password spraying (T-10) | `SecEvents \| where event == "auth.login.failure" \| summarize n = count(), accounts = dcount(id) by ip, bin(TimeGenerated, 5m) \| where n > 20 or accounts > 10` | 1 сработване | проверка; блокиране на IP при нужда |
| D2 | Brute force срещу един акаунт (T-10) | `SecEvents \| where event == "auth.login.failure" \| summarize n = count() by id, bin(TimeGenerated, 15m) \| where n > 10` | 1 | известие до собственика на акаунта |
| D3 | Сондиране за IDOR (T-01) | `SecEvents \| where event == "access.denied" \| summarize n = count(), paths = dcount(tostring(e.pathTemplate)) by userId, bin(TimeGenerated, 10m) \| where n > 30` | 1 | преглед на акаунта; деактивиране при умисъл |
| D4 | Масова регистрация (T-04) | `SecEvents \| where event == "account.registered" \| summarize n = count() by ip, bin(TimeGenerated, 10m) \| where n > 5` | 1 | проверка за заети чужди факултетни номера |
| D5 | Задържане на места (T-07) | `SecEvents \| where event startswith "enrollment." \| summarize n = count() by userId, bin(TimeGenerated, 5m) \| where n > 10` | 1 | проверка; ограничаване |
| D6 | Повишаване на права (T-03) | `SecEvents \| where event == "account.role_changed" and tostring(e.to) in ("ADMIN","INSPECTOR")` | всяко | потвърждение от админ |
| D7 | Промени извън работно време (T-09) | `SecEvents \| where event in ("account.enabled_changed","student.status_changed") \| where hourofday(TimeGenerated + 2h) !between (7 .. 19)` | всяко | преглед |
| D8 | Наличност в 08:00 (T-16) | `ContainerAppConsoleLogs_CL \| where Log_s has "\"status\":5"` или метриката `Requests` по `statusCodeCategory = 5xx` | > 0,5 % за 5 минути | ескалация към DevOps |

Праговете са начални и се настройват след първата кампания.

## 3. План за реакция при инцидент

### Роли

| Роля | Кой | Отговаря за |
| --- | --- | --- |
| Ръководител на инцидента | Security Engineer (Алекс) | координация, решения, хронология |
| DevOps | DevOps инженерът на екипа | Azure, rollback, ротация на тайните, логове |
| Комуникация | Ръководителят на екипа | Ректорат, другите екипи, потребителите |
| Лични данни | DPO на университета | оценка дали има нарушение и уведомяване на КЗЛД |

### Стъпки

1. **Откриване и оценка (до 30 мин.).** Какво се случва, откога, кои акаунти и данни са засегнати, продължава ли. Тежест: S1 (изтичане на лични данни или тайни, превземане на акаунт със служебна роля), S2 (засегнат е един студент или функция), S3 (опит без успех).
2. **Ограничаване.** Деактивиране на засегнатите акаунти (`users.enabled = false`), отнемане на сесиите и токените, затваряне на курс, блокиране на IP на ingress (Container Apps IP restrictions, безплатно), rollback чрез revert PR или директно към предишен образ (README, „Checking and rolling back production“).
3. **Ротация на засегнатите тайни.** MySQL паролата → нов `DATABASE_URL` в GitHub **и** в Container App secret `db-url` (`az containerapp secret set`, после нова ревизия; `deploy` не го прави сам, H-04); `GHCR_PAT`; ключовете за подписване; проверка на federated credentials на `unicloud-github-deployer`.
4. **Запазване на доказателства.** Експорт на логовете от Log Analytics преди изтичане на срока им; време в UTC; бележки за всяко действие.
5. **Отстраняване.** Поправка с регресионен тест, `sec-sweep --scope diff`, преглед и сливане.
6. **Уведомяване.**
   - Ако има засегнати лични данни: незабавно към DPO. Университетът уведомява **КЗЛД до 72 часа** след установяването (чл. 33 GDPR), а засегнатите лица, ако рискът за тях е висок (чл. 34).
   - Другите екипи на UniCloud, ако са засегнати идентичности или записвания, които те ползват.
   - Ректоратът при S1 или при прекъсване по време на кампания.
7. **Преглед след инцидента (до 1 седмица).** Без търсене на виновни. Обновяване на T-xx, SR-xx и V-xx.

### Шаблон за хронология

| Време (UTC) | Кой | Какво е наблюдавано или направено |
| --- | --- | --- |
| | | |
