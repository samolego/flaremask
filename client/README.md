# Flaremask – Client

This directory contains the Svelte 5 web frontend and the Firefox MV3 browser extension.

## Requirements

| Tool | Minimum version |
|------|----------------|
| Node.js | 20.x |
| npm | 10.x |

## Install dependencies

```bash
npm install
```

## Build the Firefox extension

```bash
npm run build:ext
```

The built extension is output to `dist-ext/`. Load it in Firefox via `about:debugging` → *Load Temporary Add-on* → select `dist-ext/manifest.json`, or package it with:

```bash
cd dist-ext && zip -r ../flaremask.zip .
```

## Build the web frontend

```bash
npm run build
```

Output goes to `dist/`.

## Development

```bash
npm run dev
```

Starts a local dev server at `http://localhost:5173` with hot reload.
The dev server proxies `/api` and `/auth` to `http://localhost:8787` (the Cloudflare Worker).
