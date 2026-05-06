use serde::{Deserialize, Serialize};
use worker::{Fetch, Headers, Method, Request, RequestInit};

const CF_API: &str = "https://api.cloudflare.com/client/v4";
const CF_PAGE_SIZE: u32 = 50;

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    result: Option<T>,
    success: bool,
    errors: Vec<ApiMessage>,
}

#[derive(Debug, Deserialize)]
struct ApiListResponse<T> {
    result: Option<Vec<T>>,
    success: bool,
    errors: Vec<ApiMessage>,
    result_info: Option<ResultInfo>,
}

#[derive(Debug, Deserialize)]
struct ResultInfo {
    total_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct ApiMessage {
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    #[serde(skip_serializing)]
    pub id: String,
    pub name: String,
    pub matchers: Vec<Matcher>,
    pub actions: Vec<Action>,
    pub enabled: bool,
    pub priority: i32,
}

impl Rule {
    /// Returns true if one of the forward targets is "email".
    pub fn owned_by(&self, email: &str) -> bool {
        self.actions.iter().any(|a| match a {
            Action::Forward { value } => value.iter().any(|v| v == email),
            _ => false,
        })
    }

    /// The "to" alias address this rule matches, if it is a literal `to` matcher.
    pub fn alias_address(&self) -> Option<&str> {
        self.matchers.iter().find_map(|m| match m {
            Matcher::Literal { field, value } if field == "to" => Some(value.as_str()),
            _ => None,
        })
    }
}

/// Allows managing masks even if the user logs in via an existing alias.
/// Resolves the canonical "real" email for "oidc_email" given all zone rules,
/// and returns the root alias rule id (if any).
pub fn resolve_ownership<'a>(rules: &'a [Rule], oidc_email: &'a str) -> (&'a str, Option<&'a str>) {
    for rule in rules {
        if rule.alias_address() == Some(oidc_email) {
            // if login is an alias, we must treat the target inbox as the owner
            if let Some(Action::Forward { value }) = rule
                .actions
                .iter()
                .find(|a| matches!(a, Action::Forward { .. }))
            {
                if let Some(real) = value.first() {
                    return (real.as_str(), Some(rule.id.as_str()));
                }
            }
        }
    }
    (oidc_email, None)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Matcher {
    #[serde(rename = "literal")]
    Literal { field: String, value: String },
    #[serde(rename = "all")]
    All,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "forward")]
    Forward { value: Vec<String> },
    #[serde(rename = "drop")]
    Drop,
}

#[derive(Debug, Serialize)]
pub struct EmailRule {
    pub id: String,
    pub alias: String,
    pub name: String,
    pub enabled: bool,
    pub is_root: bool,
}

impl TryFrom<Rule> for EmailRule {
    type Error = String;
    fn try_from(rule: Rule) -> Result<Self, String> {
        let alias = rule
            .matchers
            .iter()
            .find_map(|m| match m {
                Matcher::Literal { field, value } if field == "to" => Some(value.clone()),
                _ => None,
            })
            .ok_or_else(|| format!("Rule {} has no 'to' matcher", rule.id))?;
        Ok(EmailRule {
            id: rule.id,
            alias,
            name: rule.name,
            enabled: rule.enabled,
            is_root: false,
        })
    }
}

fn cf_headers(token: &str) -> Result<Headers, String> {
    let headers = Headers::new();
    headers
        .set("Authorization", &format!("Bearer {token}"))
        .map_err(|e| e.to_string())?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| e.to_string())?;
    Ok(headers)
}

pub async fn get_zone_domain(token: &str, zone_id: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct Zone {
        name: String,
    }

    let url = format!("{CF_API}/zones/{zone_id}");
    let mut init = RequestInit::new();
    init.with_method(Method::Get)
        .with_headers(cf_headers(token)?);
    let req = Request::new_with_init(&url, &init).map_err(|e| e.to_string())?;
    let mut resp = Fetch::Request(req)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let body: ApiResponse<Zone> = resp.json().await.map_err(|e| e.to_string())?;
    if !body.success {
        let msg: Vec<&str> = body
            .errors
            .iter()
            .filter_map(|e| e.message.as_deref())
            .collect();
        return Err(format!("CF API error: {}", msg.join("; ")));
    }
    Ok(body.result.ok_or("CF API returned null zone")?.name)
}

pub async fn list_rules(token: &str, zone_id: &str) -> Result<Vec<Rule>, String> {
    let mut all_rules: Vec<Rule> = Vec::new();
    let mut page = 1u32;

    // Full zone view is required to reliably verify rule ownership and resolve aliases.
    loop {
        let url = format!(
            "{CF_API}/zones/{zone_id}/email/routing/rules?per_page={CF_PAGE_SIZE}&page={page}"
        );
        let mut init = RequestInit::new();
        init.with_method(Method::Get)
            .with_headers(cf_headers(token)?);
        let req = Request::new_with_init(&url, &init).map_err(|e| e.to_string())?;
        let mut resp = Fetch::Request(req)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: ApiListResponse<Rule> = resp.json().await.map_err(|e| e.to_string())?;
        if !body.success {
            let msg: Vec<&str> = body
                .errors
                .iter()
                .filter_map(|e| e.message.as_deref())
                .collect();
            return Err(format!("CF API error: {}", msg.join("; ")));
        }
        all_rules.extend(body.result.unwrap_or_default());

        let total = body
            .result_info
            .as_ref()
            .map(|i| i.total_count)
            .unwrap_or(0);
        if all_rules.len() as u32 >= total {
            break;
        }
        page += 1;
    }

    Ok(all_rules)
}

pub async fn create_rule(
    token: &str,
    zone_id: &str,
    alias: &str,
    forward_to: &str,
    name: &str,
) -> Result<Rule, String> {
    let url = format!("{CF_API}/zones/{zone_id}/email/routing/rules");
    let rule_body = serde_json::json!({
        "name": name,
        "enabled": true,
        "matchers": [{"type": "literal", "field": "to", "value": alias}],
        "actions": [{"type": "forward", "value": [forward_to]}],
        "priority": 0,
    });

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(cf_headers(token)?)
        .with_body(Some(wasm_bindgen::JsValue::from_str(
            &rule_body.to_string(),
        )));
    let req = Request::new_with_init(&url, &init).map_err(|e| e.to_string())?;
    let mut resp = Fetch::Request(req)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let body: ApiResponse<Rule> = resp.json().await.map_err(|e| e.to_string())?;
    if !body.success {
        let msg: Vec<&str> = body
            .errors
            .iter()
            .filter_map(|e| e.message.as_deref())
            .collect();
        return Err(format!("CF API error: {}", msg.join("; ")));
    }
    Ok(body.result.ok_or("CF API returned null result")?)
}

/// PUT an already-mutated rule back. Caller fetches via `get_rule`, patches
/// allowed fields (name / enabled), then passes the rule here.
pub async fn put_rule(
    token: &str,
    zone_id: &str,
    rule_id: &str,
    rule: Rule,
) -> Result<Rule, String> {
    let url = format!("{CF_API}/zones/{zone_id}/email/routing/rules/{rule_id}");
    let json_body = serde_json::to_string(&rule).map_err(|e| e.to_string())?;

    let mut init = RequestInit::new();
    init.with_method(Method::Put)
        .with_headers(cf_headers(token)?)
        .with_body(Some(wasm_bindgen::JsValue::from_str(&json_body)));
    let req = Request::new_with_init(&url, &init).map_err(|e| e.to_string())?;
    let mut resp = Fetch::Request(req)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let body: ApiResponse<Rule> = resp.json().await.map_err(|e| e.to_string())?;
    if !body.success {
        let msg: Vec<&str> = body
            .errors
            .iter()
            .filter_map(|e| e.message.as_deref())
            .collect();
        return Err(format!("CF API error: {}", msg.join("; ")));
    }
    Ok(body.result.ok_or("CF API returned null result")?)
}

pub async fn delete_rule(token: &str, zone_id: &str, rule_id: &str) -> Result<(), String> {
    let url = format!("{CF_API}/zones/{zone_id}/email/routing/rules/{rule_id}");
    let mut init = RequestInit::new();
    init.with_method(Method::Delete)
        .with_headers(cf_headers(token)?);
    let req = Request::new_with_init(&url, &init).map_err(|e| e.to_string())?;
    let resp = Fetch::Request(req)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status_code();
    if !(200..300).contains(&status) {
        return Err(format!("CF API delete failed with status {status}"));
    }
    Ok(())
}
