//! The OpenAPI file matches the portal: every documented path is routed, every route a module calls is
//! documented, every $ref resolves, and production hides the staging tools.

mod common;

use common::*;
use serde_json::Value;
use unicloud_portal::config::Environment;

/// Every route a module may call, as written in api/openapi.json. Add a route here and to the file together.
const MODULE_ROUTES: [&str; 16] = [
    "/.well-known/openid-configuration",
    "/oauth2/jwks",
    "/oauth2/token",
    "/userinfo",
    "/api/v1/directory/users/{sub}",
    "/api/v1/directory/users/{sub}/enrollments",
    "/api/v1/directory/users/{sub}/offerings",
    "/api/v1/directory/offerings/{id}/students",
    "/api/v1/directory/faculties/{code}/offerings",
    "/api/v1/directory/faculties/{code}/students",
    "/api/v1/staging/roles",
    "/api/v1/staging/users",
    "/api/v1/staging/users/{id}/permissions",
    "/api/v1/staging/sso-clients",
    "/health",
    "/openapi.json",
];

fn resolve<'a>(spec: &'a Value, r: &str) -> Option<&'a Value> {
    spec.pointer(r.strip_prefix('#')?)
}

fn check_refs(spec: &Value, v: &Value) {
    match v {
        Value::Object(m) => {
            if let Some(Value::String(r)) = m.get("$ref") {
                assert!(resolve(spec, r).is_some(), "broken $ref {r}");
            }
            m.values().for_each(|x| check_refs(spec, x));
        }
        Value::Array(a) => a.iter().for_each(|x| check_refs(spec, x)),
        _ => {}
    }
}

#[tokio::test]
async fn spec_and_routes_agree() {
    let p = start(Environment::Staging, &[]).await;
    let b = Browser::new(&p.base);
    let r = b.get_with("/openapi.json", &[]).await;
    assert_eq!(r.status, 200);
    let spec = r.json();
    assert_eq!(spec["openapi"], "3.1.0");
    assert_eq!(spec["servers"][0]["url"], p.base);
    check_refs(&spec, &spec);

    let paths = spec["paths"].as_object().unwrap();
    for route in MODULE_ROUTES.iter().filter(|r| **r != "/openapi.json") {
        assert!(paths.contains_key(*route), "{route} is not documented in api/openapi.json");
    }
    // Every documented operation is routed: without credentials it answers anything but an unrouted 404
    for (path, item) in paths {
        let concrete = path.replace("{sub}", "1").replace("{id}", "1").replace("{code}", "FCST");
        for method in item.as_object().unwrap().keys() {
            let url = format!("{}{concrete}", p.base);
            let http = reqwest::Client::new();
            let res = match method.as_str() {
                "get" => http.get(&url).send().await,
                "post" => http.post(&url).json(&serde_json::json!({})).send().await,
                "put" => http.put(&url).json(&serde_json::json!({})).send().await,
                other => panic!("unexpected method {other} for {path}"),
            }
            .unwrap();
            let status = res.status().as_u16();
            let body = res.text().await.unwrap();
            let unrouted = status == 404 && body.is_empty();
            assert!(
                !unrouted && status != 405,
                "{method} {path} is documented but not routed ({status})"
            );
        }
    }

    let page = b.get("/api-docs").await;
    assert_eq!(page.status, 200);
    assert!(page.text.contains("/assets/swagger/swagger-ui-bundle.js"));
    assert!(!page.text.contains("<script>"), "no inline script (CSP)");
}

#[tokio::test]
async fn production_spec_has_no_staging_tools() {
    let p = start(Environment::Production, &[]).await;
    let spec = Browser::new(&p.base).get_with("/openapi.json", &[]).await.json();
    let paths = spec["paths"].as_object().unwrap();
    assert!(paths.keys().all(|k| !k.starts_with("/api/v1/staging")));
    assert!(paths.contains_key("/api/v1/directory/users/{sub}"));
    assert!(spec["tags"].as_array().unwrap().iter().all(|t| t["name"] != "Staging tools"));
}
