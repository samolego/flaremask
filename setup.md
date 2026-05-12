# Flaremask Setup Guide

Flaremask is a Cloudflare Worker that lets users manage their own Cloudflare
Email Routing aliases. Users authenticate via your existing OIDC provider
(Authelia, Keycloak, etc.). Each user can only manage aliases that forward to
their own inbox.

---

## Prerequisites

- A Cloudflare account with **Email Routing** enabled on at least one zone
- An OIDC provider (Authelia v4.38+, Keycloak, or any standard OIDC server)
- The [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed and authenticated (`wrangler login`)

---

## 1. Register the OIDC Client

### Authelia

Add to `configuration.yaml` under `identity_providers.oidc.clients`:

```yaml
- client_id: "flaremask"
  client_name: "Flaremask"
  client_secret: "<hashed-secret>"   # see below
  public: false
  authorization_policy: "one_factor"   # or two_factor
  consent_mode: "implicit"
  token_endpoint_auth_method: "client_secret_post"
  require_pkce: true
  pkce_challenge_method: "S256"
  redirect_uris:
    - "https://<your-worker-url>/auth/callback"
  scopes:
    - "openid"
    - "email"
    - "profile"
  userinfo_signed_response_alg: "none"
```

Generate the secret (run once, keep both values):

```bash
SECRET=$(openssl rand -hex 32)
echo "Plain secret (save this): $SECRET"
docker run --rm authelia/authelia:latest \
  authelia crypto hash generate argon2 --password "$SECRET"
```

Paste the **hashed** value into `client_secret` above.  
Keep the **plain** value — you'll need it in Step 3.

### Keycloak

Hi, if you have keycloak set up and find this useful, please contribute to the guide!

---

## 2. Cloudflare — Gather Credentials

### Zone ID

Cloudflare Dashboard → your domain → Overview → right sidebar → **Zone ID**.

### API Token

Dashboard → My Profile → **API Tokens → Create Token**:

- Permissions:
  - `Zone` → `Email Routing Rules` → **Edit**
  - `Zone` → `Zone` → **Read** (you can skip this if you set `CF_EMAIL_DOMAIN` variable)
- Zone Resources: Include → Specific zone → your domain

---

## 4. Deploy

### Deploy to Cloudflare button

[![Deploy to Cloudflare](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/samolego/flaremask/tree/master)

If you want Cloudflare to fork and deploy the repository for you, use the
button above. Cloudflare will prompt for the public variables from
`wrangler.toml` and the secrets from `.dev.vars.example`.

Due to Cloudflare not including cargo in build env, you'll need to set your `build` command to this:
```bash
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"
cargo install cargo-generate
```
and the deploy command to
```bash
. "$HOME/.cargo/env"
npx wrangler deploy
```

After deployment finishes, copy the resulting Worker URL into your OIDC
provider's allowed redirect URIs.

### Manual Wrangler deploy

#### Setup

Edit `wrangler.toml` — fill in the public variables:

```toml
[vars]
OIDC_ISSUER_URL   = "https://auth.example.com"          # OIDC provider base URL, no trailing slash
OIDC_CLIENT_ID    = "flaremask"
OIDC_REDIRECT_URI = "https://<your-worker-url>/auth/callback"
CF_ZONE_ID        = "<zone-id from Step 2>"
TOKEN_EXPIRY      = "3600"                               # session length in seconds
# Optional
CF_EMAIL_DOMAIN   = "yourdomain.com"                     # You can also leave this out though it's recommended to set it
```

Then push the three secrets (each command prompts interactively — nothing is stored in shell history or config files):

```bash
wrangler secret put OIDC_CLIENT_SECRET   # plain client secret from Step 1
wrangler secret put CLOUDFLARE_API_TOKEN         # API token from Step 2
wrangler secret put JWT_SECRET           # any random 32+ char string, e.g.: openssl rand -hex 32
```

#### Deploy

```bash
wrangler deploy
```

The worker URL will be shown after deployment (e.g. `https://flaremask.<subdomain>.workers.dev`).  
Use that URL as `OIDC_REDIRECT_URI` and in your OIDC provider's redirect URI list.
To use a custom domain instead, add it under **Workers → your worker → Custom Domains** in the Cloudflare dashboard.

---

## 4. Verify

| URL | Expected result |
|-----|----------------|
| `GET /` | "Login" link when unauthenticated; email + logout link when logged in |
| `GET /auth/login` | Redirects to your OIDC provider |
| `GET /auth/callback` | Completes login, redirects to `/` |
| `GET /auth/logout` | Clears session, redirects to `/` |
| `GET /api/v1/emails` | `401` without session; JSON list of your aliases when logged in |

---

## Ownership model

Users can only see and modify aliases that forward to their inbox.

If a user's OIDC email address is itself a Cloudflare alias (e.g.
`john@yourdomain.com → real@gmail.com`), flaremask automatically resolves the
real destination and grants ownership of all aliases forwarding there.

The alias matching the OIDC email (the "root alias") cannot be deleted or
disabled — removing it would cut email delivery and effectively lock the user
out of your OIDC provider.
