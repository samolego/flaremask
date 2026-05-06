use axum::{
    extract::Request,
    http::{header, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use tower_service::Service;
use worker::*;

mod api;
mod auth;

#[derive(Clone)]
pub struct AppState {
    pub oidc_issuer_url: String,
    pub oidc_client_id: String,
    pub oidc_client_secret: String,
    pub oidc_redirect_uri: String,
    /// Allowed CORS origin for the API. Set to the frontend URL (Pages domain)
    /// in production. Defaults to `*` when absent.
    pub allowed_origin: Option<String>,
    pub cf_zone_id: String,
    pub cf_email_domain: Option<String>,
    pub cf_api_token: String,
    pub jwt_secret: String,
    pub token_expiry: u64,
}

impl AppState {
    fn from_env(env: &Env) -> Result<Self> {
        Ok(Self {
            oidc_issuer_url: env.var("OIDC_ISSUER_URL")?.to_string(),
            oidc_client_id: env.var("OIDC_CLIENT_ID")?.to_string(),
            oidc_redirect_uri: env.var("OIDC_REDIRECT_URI")?.to_string(),
            allowed_origin: env.var("ALLOWED_ORIGIN").ok().map(|v| v.to_string()),
            cf_zone_id: env.var("CF_ZONE_ID")?.to_string(),
            cf_email_domain: env.var("CF_EMAIL_DOMAIN").ok().map(|v| v.to_string()),
            token_expiry: env.var("TOKEN_EXPIRY")?.to_string().parse().unwrap_or(1800),
            // secrets
            oidc_client_secret: env.secret("OIDC_CLIENT_SECRET")?.to_string(),
            cf_api_token: env.secret("CLOUDFLARE_API_TOKEN")?.to_string(),
            jwt_secret: env.secret("JWT_SECRET")?.to_string(),
        })
    }
}

async fn cors_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let origin = state.allowed_origin.as_deref().unwrap_or("*").to_string();
    let is_preflight = req.method() == Method::OPTIONS;

    let mut response = if is_preflight {
        StatusCode::NO_CONTENT.into_response()
    } else {
        next.run(req).await
    };

    let headers = response.headers_mut();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.parse().unwrap());
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "GET, POST, PATCH, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "Authorization, Content-Type".parse().unwrap(),
    );
    response
}

fn router(state: AppState) -> Router {
    Router::new()
        .nest("/auth", auth::router())
        .nest(
            "/api/v1",
            api::router().layer(middleware::from_fn_with_state(
                state.clone(),
                cors_middleware,
            )),
        )
        .with_state(state)
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    let state = AppState::from_env(&env)?;
    Ok(router(state).call(req).await?)
}
