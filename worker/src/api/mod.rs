use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::AppState;

pub(crate) mod cloudflare;
use cloudflare::{resolve_ownership, EmailRule};

#[derive(Serialize)]
struct ListResponse {
    destination: String,
    aliases: Vec<EmailRule>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/emails", get(list).post(create))
        .route("/emails/{id}", patch(update).delete(remove))
}

pub async fn list(AuthUser(email): AuthUser, State(state): State<AppState>) -> impl IntoResponse {
    send_wrapper::SendWrapper::new(list_impl(email, state)).await
}

pub async fn create(
    AuthUser(email): AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateRequest>,
) -> impl IntoResponse {
    send_wrapper::SendWrapper::new(create_impl(email, state, body)).await
}

pub async fn update(
    AuthUser(email): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateRequest>,
) -> impl IntoResponse {
    send_wrapper::SendWrapper::new(update_impl(email, state, id, body)).await
}

pub async fn remove(
    AuthUser(email): AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    send_wrapper::SendWrapper::new(remove_impl(email, state, id)).await
}

async fn list_impl(oidc_email: String, state: AppState) -> impl IntoResponse {
    let rules = match cloudflare::list_rules(&state.cf_api_token, &state.cf_zone_id).await {
        Ok(r) => r,
        Err(e) => {
            worker::console_error!("list_rules error: {e}");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };

    let (real_email, root_id) = resolve_ownership(&rules, &oidc_email);
    let real_email = real_email.to_string();
    let root_id = root_id.map(str::to_string);

    let aliases: Vec<EmailRule> = rules
        .into_iter()
        .filter(|r| r.owned_by(&real_email))
        .filter_map(|r| {
            let is_root = root_id.as_deref() == Some(r.id.as_str());
            EmailRule::try_from(r).ok().map(|mut e| {
                e.is_root = is_root;
                e
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(ListResponse {
            destination: real_email,
            aliases,
        }),
    )
        .into_response()
}

async fn create_impl(
    oidc_email: String,
    state: AppState,
    body: CreateRequest,
) -> impl IntoResponse {
    if body.alias.contains('@') {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "alias must be a local part only (no @)",
        )
            .into_response();
    }

    // Resolve the real inbox to use as forward-to target
    let rules = match cloudflare::list_rules(&state.cf_api_token, &state.cf_zone_id).await {
        Ok(r) => r,
        Err(e) => {
            worker::console_error!("list_rules error: {e}");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };
    let (real_email, _) = resolve_ownership(&rules, &oidc_email);
    let real_email = real_email.to_string();

    let name = body.name.clone().unwrap_or_else(|| body.alias.clone());
    let domain = if let Some(d) = &state.cf_email_domain {
        d.clone()
    } else {
        match cloudflare::get_zone_domain(&state.cf_api_token, &state.cf_zone_id).await {
            Ok(d) => d,
            Err(e) => {
                worker::console_error!("get_zone_domain error: {e}");
                return StatusCode::BAD_GATEWAY.into_response();
            }
        }
    };
    let full_alias = format!("{}@{}", body.alias, domain);
    match cloudflare::create_rule(
        &state.cf_api_token,
        &state.cf_zone_id,
        &full_alias,
        &real_email,
        &name,
    )
    .await
    {
        Ok(rule) => match EmailRule::try_from(rule) {
            Ok(r) => (StatusCode::CREATED, Json(r)).into_response(),
            Err(e) => {
                worker::console_error!("create rule conversion error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(e) => {
            worker::console_error!("create_rule error: {e}");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

async fn update_impl(
    oidc_email: String,
    state: AppState,
    id: String,
    body: UpdateRequest,
) -> impl IntoResponse {
    let rules = match cloudflare::list_rules(&state.cf_api_token, &state.cf_zone_id).await {
        Ok(r) => r,
        Err(e) => {
            worker::console_error!("list_rules error: {e}");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };

    let (real_email, root_id) = resolve_ownership(&rules, &oidc_email);
    let real_email = real_email.to_string();
    let root_id = root_id.map(str::to_string);

    // Prevents accidental lockout by protecting the link between login email and inbox.
    if root_id.as_deref() == Some(id.as_str()) {
        return (StatusCode::FORBIDDEN, "cannot modify the root alias").into_response();
    }

    let mut rule = match rules.into_iter().find(|r| r.id == id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    if !rule.owned_by(&real_email) {
        return StatusCode::FORBIDDEN.into_response();
    }

    if let Some(enabled) = body.enabled {
        rule.enabled = enabled;
    }
    if let Some(name) = body.name {
        rule.name = name;
    }

    match cloudflare::put_rule(&state.cf_api_token, &state.cf_zone_id, &id, rule).await {
        Ok(updated) => match EmailRule::try_from(updated) {
            Ok(r) => (StatusCode::OK, Json(r)).into_response(),
            Err(e) => {
                worker::console_error!("update rule conversion error: {e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(e) => {
            worker::console_error!("put_rule error: {e}");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

async fn remove_impl(oidc_email: String, state: AppState, id: String) -> impl IntoResponse {
    let rules = match cloudflare::list_rules(&state.cf_api_token, &state.cf_zone_id).await {
        Ok(r) => r,
        Err(e) => {
            worker::console_error!("list_rules error: {e}");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };

    let (real_email, root_id) = resolve_ownership(&rules, &oidc_email);
    let real_email = real_email.to_string();
    let root_id = root_id.map(str::to_string);

    // Prevents accidental lockout by protecting the link between login email and inbox.
    if root_id.as_deref() == Some(id.as_str()) {
        return (StatusCode::FORBIDDEN, "cannot delete the root alias").into_response();
    }

    let rule = match rules.into_iter().find(|r| r.id == id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    if !rule.owned_by(&real_email) {
        return StatusCode::FORBIDDEN.into_response();
    }

    match cloudflare::delete_rule(&state.cf_api_token, &state.cf_zone_id, &id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            worker::console_error!("delete_rule error: {e}");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct CreateRequest {
    pub alias: String,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub enabled: Option<bool>,
    pub name: Option<String>,
}
