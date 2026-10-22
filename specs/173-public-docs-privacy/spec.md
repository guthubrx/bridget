# 173 — Public documentation and repository privacy

Status: Implemented
Date: 2026-10-23

## Authorized scope

Audit the Git repository for secrets and privacy-sensitive material. Make English
the default language for the README and public documentation. The user chose to
retain French editions and leave specifications/historical records in their
original language.

## Acceptance

- Current files and available Git history are scanned without exposing secrets.
- Findings are classified; concrete operational metadata is sanitized in current files.
- Historical exposure and unexamined surfaces are explicitly documented.
- Public entry pages are English and provide French links.
- Existing French guides remain available; archived records are clearly identified.
- New changes receive a repeatable secret check and local-link validation.
- Delivery does not restart services, change account rights or rewrite Git history.
