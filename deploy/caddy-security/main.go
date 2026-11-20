package main

import (
	caddycmd "github.com/caddyserver/caddy/v2/cmd"

	// Standard reverse proxy, HTTP, TLS (ACME public via caddytls/certmagic), logging and metrics modules.
	// caddypki / acmeserver (local CA via smallstep/go-jose/v3) are intentionally omitted to eliminate
	// unmaintained go-jose/v3 CVEs and >100 unnecessary indirect dependencies.
	_ "github.com/caddyserver/caddy/v2/caddyconfig/caddyfile"
	_ "github.com/caddyserver/caddy/v2/modules/caddyevents"
	_ "github.com/caddyserver/caddy/v2/modules/caddyevents/eventsconfig"
	_ "github.com/caddyserver/caddy/v2/modules/caddyfs"
	_ "github.com/caddyserver/caddy/v2/modules/caddyhttp/standard"
	_ "github.com/caddyserver/caddy/v2/modules/caddytls"
	_ "github.com/caddyserver/caddy/v2/modules/caddytls/distributedstek"
	_ "github.com/caddyserver/caddy/v2/modules/caddytls/standardstek"
	_ "github.com/caddyserver/caddy/v2/modules/filestorage"
	_ "github.com/caddyserver/caddy/v2/modules/logging"
	_ "github.com/caddyserver/caddy/v2/modules/metrics"
)

func main() { caddycmd.Main() }
