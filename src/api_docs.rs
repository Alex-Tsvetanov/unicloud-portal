//! `/openapi.json` (api/openapi.json, with this portal's address and the staging tools only where they
//! exist) and `/api-docs`, a Swagger UI page served from /assets/swagger.

use std::sync::OnceLock;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use maud::{DOCTYPE, Markup, html};
use serde_json::{Value, json};

use crate::oidc::issuer;
use crate::state::AppState;

const SPEC: &str = include_str!("../api/openapi.json");
const STAGING_TAG: &str = "Staging tools";

fn base_spec() -> &'static Value {
    static S: OnceLock<Value> = OnceLock::new();
    S.get_or_init(|| serde_json::from_str(SPEC).expect("api/openapi.json is valid JSON (tested)"))
}

/// The spec for this portal: its own address as the server, and no staging routes in production
pub fn spec_for(base: &str, staging_tools: bool) -> Value {
    let mut spec = base_spec().clone();
    spec["servers"] = json!([{ "url": base }]);
    if let Some(flows) = spec.pointer_mut("/components/securitySchemes/directoryToken/flows/clientCredentials") {
        flows["tokenUrl"] = json!(format!("{base}/oauth2/token"));
    }
    if !staging_tools {
        if let Some(paths) = spec["paths"].as_object_mut() {
            paths.retain(|_, item| {
                !item
                    .as_object()
                    .into_iter()
                    .flat_map(|ops| ops.values())
                    .any(|op| has_tag(op, STAGING_TAG))
            });
        }
        if let Some(tags) = spec["tags"].as_array_mut() {
            tags.retain(|t| t["name"] != STAGING_TAG);
        }
    }
    spec
}

fn has_tag(op: &Value, tag: &str) -> bool {
    op["tags"].as_array().is_some_and(|tags| tags.iter().any(|t| t == tag))
}

pub async fn openapi(State(st): State<AppState>, headers: HeaderMap) -> Json<Value> {
    Json(spec_for(&issuer(&st, &headers), st.cfg.staging_tools()))
}

pub async fn page() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "UniCloud Portal API" }
                link rel="stylesheet" href="/assets/swagger/swagger-ui.css";
                link rel="stylesheet" href="/assets/swagger/page.css";
                meta name="color-scheme" content="light";
            }
            body {
                div #swagger-ui {}
                script src="/assets/swagger/swagger-ui-bundle.js" {}
                script src="/assets/swagger/init.js" {}
            }
        }
    }
}
