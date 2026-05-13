use axum::{
    extract::{FromRef, FromRequestParts, Query, State},
    http::{header::SET_COOKIE, request::Parts, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::AppState;

const PKCE_COOKIE: &str = "pkce";

#[derive(Serialize, Deserialize)]
pub struct SessionClaims {
    pub email: String,
}

pub struct AuthUser(pub String);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let token = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;

        let key = HS256Key::from_bytes(app_state.jwt_secret.as_bytes());
        let claims = key
            .verify_token::<SessionClaims>(token, None)
            .map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;

        Ok(AuthUser(claims.custom.email))
    }
}

pub fn get_cookie(header: &str, name: &str) -> Option<String> {
    header.split(';').find_map(|part| {
        let (k, v) = part.trim().split_once('=')?;
        (k.trim() == name).then(|| v.trim().to_string())
    })
}

fn set_cookie(name: &str, value: &str, max_age: i64) -> String {
    format!(
        "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
        name, value, max_age
    )
}

#[derive(Deserialize)]
struct OidcEndpoints {
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: String,
}

async fn fetch_oidc_endpoints(issuer_url: &str) -> Result<OidcEndpoints, String> {
    use worker::{Fetch, Method, Request, RequestInit};

    let discovery_url = format!(
        "{}/.well-known/openid-configuration",
        issuer_url.trim_end_matches('/')
    );
    let mut init = RequestInit::new();
    init.with_method(Method::Get);
    let req = Request::new_with_init(&discovery_url, &init).map_err(|e| e.to_string())?;
    let mut resp = Fetch::Request(req)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<OidcEndpoints>()
        .await
        .map_err(|e| format!("OIDC discovery failed: {e}"))
}

pub async fn login(State(state): State<AppState>, Query(params): Query<LoginQuery>) -> Response {
    send_wrapper::SendWrapper::new(login_impl(state, params.return_to)).await
}

#[derive(Deserialize)]
pub struct LoginQuery {
    /// Optional post-auth redirect for browser extensions (allizom / chromiumapp only).
    return_to: Option<String>,
}

fn is_extension_return_url(url: &str) -> bool {
    url.starts_with("https://") && {
        let host = url
            .trim_start_matches("https://")
            .split('/')
            .next()
            .unwrap_or("");
        host.ends_with(".extensions.allizom.org") || host.ends_with(".chromiumapp.org")
    }
}

async fn login_impl(state: AppState, return_to: Option<String>) -> Response {
    let endpoints = match fetch_oidc_endpoints(&state.oidc_issuer_url).await {
        Ok(e) => e,
        Err(e) => {
            worker::console_error!("OIDC discovery failed: {e}");
            return (StatusCode::BAD_GATEWAY, "OIDC discovery failed").into_response();
        }
    };

    let mut verifier_bytes = [0u8; 32];
    getrandom::fill(&mut verifier_bytes).expect("rng failed");
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));

    let mut state_bytes = [0u8; 16];
    getrandom::fill(&mut state_bytes).expect("rng failed");
    let oauth_state = URL_SAFE_NO_PAD.encode(state_bytes);

    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}&scope=openid+email",
        endpoints.authorization_endpoint,
        urlencoding::encode(&state.oidc_client_id),
        urlencoding::encode(&state.oidc_redirect_uri),
        code_challenge,
        oauth_state,
    );

    // Embed validated return_to in PKCE cookie (empty = use default frontend redirect).
    let return_to_value = return_to
        .filter(|u| is_extension_return_url(u))
        .unwrap_or_default();
    // Combined to reduce cookie overhead and ensure atomic expiration.
    let pkce_value = format!("{code_verifier}:{oauth_state}:{return_to_value}");
    (
        [(SET_COOKIE, set_cookie(PKCE_COOKIE, &pkce_value, 300))],
        Redirect::to(&auth_url),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
}

pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackQuery>,
    req: axum::extract::Request,
) -> Response {
    let cookie_str = req
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    send_wrapper::SendWrapper::new(callback_impl(state, params, cookie_str)).await
}

async fn callback_impl(state: AppState, params: CallbackQuery, cookie_str: String) -> Response {
    if let Some(err) = params.error {
        return (StatusCode::BAD_REQUEST, format!("OIDC error: {err}")).into_response();
    }
    let code = match params.code {
        Some(c) => c,
        None => return (StatusCode::BAD_REQUEST, "Missing code").into_response(),
    };

    let pkce_value = match get_cookie(&cookie_str, PKCE_COOKIE) {
        Some(v) => v,
        None => return (StatusCode::BAD_REQUEST, "Missing PKCE cookie").into_response(),
    };
    // Format: "{code_verifier}:{oauth_state}:{return_to}" (return_to may be empty)
    let mut parts = pkce_value.splitn(3, ':');
    let code_verifier = match parts.next() {
        Some(v) => v.to_string(),
        None => return (StatusCode::BAD_REQUEST, "Invalid PKCE cookie").into_response(),
    };
    let expected_state = match parts.next() {
        Some(v) => v.to_string(),
        None => return (StatusCode::BAD_REQUEST, "Invalid PKCE cookie").into_response(),
    };
    let return_to = parts.next().unwrap_or("").to_string();

    let received_state = match params.state.as_deref() {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "Missing state parameter").into_response(),
    };
    if received_state != expected_state {
        return (StatusCode::BAD_REQUEST, "State mismatch").into_response();
    }

    let endpoints = match fetch_oidc_endpoints(&state.oidc_issuer_url).await {
        Ok(e) => e,
        Err(e) => {
            worker::console_error!("OIDC discovery failed: {e}");
            return (StatusCode::BAD_GATEWAY, "OIDC discovery failed").into_response();
        }
    };

    let email = match fetch_user_email(&state, &endpoints, &code, &code_verifier).await {
        Ok(e) => e,
        Err(e) => {
            worker::console_error!("Authentication failed: {e}");
            return (StatusCode::BAD_GATEWAY, "Authentication failed").into_response();
        }
    };

    let key = HS256Key::from_bytes(state.jwt_secret.as_bytes());
    let claims = Claims::with_custom_claims(
        SessionClaims { email },
        Duration::from_secs(state.token_expiry),
    );
    let session_token = match key.authenticate(claims) {
        Ok(t) => t,
        Err(e) => {
            worker::console_error!("JWT signing failed: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Use extension return_to if provided, otherwise fall back to the root (served by Cloudflare assets).
    let redirect_url = if !return_to.is_empty() {
        format!("{return_to}#token={session_token}")
    } else {
        format!("/#token={session_token}")
    };
    let mut response = Redirect::to(&redirect_url).into_response();
    response
        .headers_mut()
        .append(SET_COOKIE, set_cookie(PKCE_COOKIE, "", 0).parse().unwrap());
    response
}

pub async fn logout() -> impl IntoResponse {
    Redirect::to("/")
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
}

/// Obtain the user's email via UserInfo. Portability is prioritized over
/// token-parsing as OIDC providers vary in ID token claims.
async fn fetch_user_email(
    state: &AppState,
    endpoints: &OidcEndpoints,
    code: &str,
    code_verifier: &str,
) -> Result<String, String> {
    use wasm_bindgen::JsValue;
    use worker::{Fetch, Headers, Method, Request, RequestInit};

    let token_body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}&code_verifier={}",
        urlencoding::encode(code),
        urlencoding::encode(&state.oidc_redirect_uri),
        urlencoding::encode(&state.oidc_client_id),
        urlencoding::encode(&state.oidc_client_secret),
        urlencoding::encode(code_verifier),
    );

    let token_headers = Headers::new();
    token_headers
        .set("Content-Type", "application/x-www-form-urlencoded")
        .map_err(|e| e.to_string())?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(token_headers)
        .with_body(Some(JsValue::from_str(&token_body)));

    let req =
        Request::new_with_init(&endpoints.token_endpoint, &init).map_err(|e| e.to_string())?;
    let mut resp = Fetch::Request(req)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let token_json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = token_json["error"].as_str() {
        return Err(format!("token endpoint error: {err}"));
    }
    let access_token = token_json["access_token"]
        .as_str()
        .ok_or("access_token missing in token response")?
        .to_string();

    let userinfo_headers = Headers::new();
    userinfo_headers
        .set("Authorization", &format!("Bearer {access_token}"))
        .map_err(|e| e.to_string())?;

    let mut init2 = RequestInit::new();
    init2
        .with_method(Method::Get)
        .with_headers(userinfo_headers);

    let req2 =
        Request::new_with_init(&endpoints.userinfo_endpoint, &init2).map_err(|e| e.to_string())?;
    let mut resp2 = Fetch::Request(req2)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let userinfo: serde_json::Value = resp2.json().await.map_err(|e| e.to_string())?;

    userinfo["email"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "email claim not found in userinfo".to_string())
}
