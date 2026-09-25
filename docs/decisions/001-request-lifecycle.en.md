# ADR 001 — Model Bridget requests as a persistent lifecycle

[🇫🇷 Français](001-cycle-vie-demandes.md) · **🇬🇧 English**

**Date**: 2026-08-14
**Status**: Accepted

## Context

A request expecting an answer currently creates in-memory reminders. If
priorities change, the sender can neither stop the reminders nor release the
recipient from the obligation to answer. A restart can also lose the state of
that pending wait.

## Decision

A tracked request receives an identifier and a durable state. Its sender can
cancel it idempotently. Terminal states are never reopened and no reminder ever
applies to them.

Ordinary messages remain plain notifications: they create no task, no state and
no tracking cost.

## Consequences

- Coordination becomes explicit: an answer is required only for a request that is
  still open.
- The local database gains a small additional registry, reusing the existing
  SQLite storage.
- Cancellation does not promise to stop an AI model already at work; it stops the
  obligation to answer and the Bridget reminders.
- Future work can add cooperative interruption without changing the base
  semantics.
