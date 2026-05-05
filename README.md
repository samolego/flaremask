# Flaremask

A self-hosted email alias manager built as a Cloudflare Worker. It provides a simple API and web interface to manage Cloudflare Email Routing rules, allowing you to create and delete email masks on the fly.

[![Deploy to Cloudflare](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/samolego/flaremask)


(See [setup](./setup.md) for detailed instructions)

## Features

- **OIDC Authentication**: Integration with OIDC provider for secure access.
- **Ownership Enforcement**: Users can only manage aliases that forward to their authenticated email address.
- **Root Alias Protection**: Prevents accidental deletion of the primary alias linking your login to your inbox.

## Prerequisites

1.  A domain on Cloudflare with **Email Routing** enabled.
2.  A Cloudflare **API Token** with `Email Routing: Edit` permissions.
3.  An **OIDC Provider** (like Authelia, Kanidm, or Authentik).

## Configuration

Edit `wrangler.toml` to set your environment variables:

| Variable | Description |
| :--- | :--- |
| `OIDC_ISSUER_URL` | Base URL of your OIDC provider. |
| `OIDC_CLIENT_ID` | The Client ID registered in your provider. |
| `OIDC_REDIRECT_URI` | Your worker's callback URL (e.g., `https://mask.example.com/auth/callback`). |
| `CF_ZONE_ID` | The ID of the Cloudflare Zone managing your domain. |
| `TOKEN_EXPIRY` | Session duration in seconds (default: 3600). |
| `CF_EMAIL_DOMAIN` | Optional (though recommended). Your cloudflare domain. |

### Secrets

Set the following secrets using `wrangler secret put <NAME>`:

- `OIDC_CLIENT_SECRET`: The secret provided by your OIDC provider.
- `CF_API_TOKEN`: Your Cloudflare API token.
- `JWT_SECRET`: A random string used to sign session tokens (min 32 chars).
