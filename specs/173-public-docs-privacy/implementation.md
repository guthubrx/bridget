# Implementation record

Baseline: 320283ef770b6d3a58569c62b02c6153145df722.

Gitleaks 8.30.1 was downloaded from its official release with archive SHA-256
verification. All 82 findings were reviewed: digests, Git IDs, synthetic test
values/request IDs and the standard WebSocket example nonce. Exact historical
fingerprints are retained, without directory/rule allowlists.

Two operational IPs, one SSH hostname and a personal email were replaced in
current documents/fixtures. Two absolute node_modules links were removed.
Historical commits and normal Git authorship remain unchanged.

52 current PNGs were OCRed locally and scanned without a finding. A representative
native screenshot was inspected visually. Two compressed artifacts were covered
by the archive-aware pass. This is not exhaustive visual review of all historical
images or an audit of GitHub attachments.

Public English guides are organized by user workflow rather than copied session
logs. Original French guides remain beside them. English indexes label the
historical French material. Agent instruction files remain in their original
language and are explicitly marked as such.

Validation so far:
- Secret history gate: no untriaged finding after the exact historical exclusions.
- Staged diff: no finding.
- Documentation: 27 pages, 421 local links and 11 language pairs pass.
- A synthetic token with an inline allow comment is still detected.
- A deliberately broken local link fails validation, then passes after restoration.
- No application build or runtime suite: no production behavior changed.

Private raw reports and OCR outputs are outside Git in the operator's private
cache. The public audit documents scope and limitations without copying their
raw contents. Independent review and publication are the remaining steps.
