# 177 — Independent mobile access to a Bridget federation

Date: 2026-10-24
Status: Accepted design; deployment pending
[English](177-mobile-federation.md) · [Français](177-mobile-federation.fr.md)

## Context

A packaged Android app must control agents across several machines, including when a desktop machine is offline. The existing federation has one authority and durable message/task receipts. Some permission checks depend on the provider runtime on its original machine.

## Decision

Build an original Android interface and a scoped, revocable human-device access layer. Reuse Bridget's directory, canonical project identities, message receipts, task controls and journals. Publish a loopback-only access component through the existing HTTPS reverse proxy on a dedicated subdomain.

Keep one federation authority. Prepare a migration to an always-on server. Preserve permission validation on the original runtime host through an authenticated, negotiated origin binding before enabling that migration. A proxy alone does not establish desktop independence.

Device authorization is explicit. It never impersonates an agent or manufactures a provider permission policy. The existing task admission checks remain authoritative.

## Consequences

There is one business engine and no replicated task database. The new code owns device authentication, the HTTP boundary and Android state. Operations need bounded payloads, scoped authorization and truthful uncertain outcomes. The migration remains blocked until authority-preservation tests and recovery checks pass.

The first implementation targets Android. Automatic authority failover, a full IDE and other mobile platforms remain separate work.

### Scoped immediate message delivery

The paired device's existing Message permission authorizes an optional
`guide_now` message only after device, machine, project and recipient checks.
This proof is passed internally for that send alone. It does not turn the socket
into an administrator, impersonate an agent, or authorize interrupt/restart.
The message intent and fallback marker belong to the original idempotent canon.
A carrier may downgrade only its delivery copy to its ordinary queue. The ID,
issued time, body and durable receipt remain unchanged. Strict non-mobile steer
requests keep their existing permission checks and refusal behavior.
