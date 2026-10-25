# Repository security and privacy review — 2026-10-23

[English](security-audit.md) · [Français](security-audit.fr.md) · [Security policy](../SECURITY.md)

## Result

**No real API credential or private key was identified by the completed checks.**
This is a bounded audit result, not a guarantee that every sensitive item is absent.

Gitleaks 8.30.1 reported 82 historical findings. Each was examined in context.
They were file/build digests, Git object identifiers, synthetic fixture values,
test request identifiers or the standard WebSocket example nonce. The checked-in
`.gitleaksignore` lists their exact immutable commit/path/rule/line fingerprints.
No entire directory or secret rule is excluded.

Actual operational metadata was found: two public deployment IP addresses,
an SSH hostname and a personal email address in historical documents/fixtures.
The current files replace them with documentation examples. Two absolute
`node_modules` symlinks exposing a workstation dependency path were removed.
Generic account/host names alone were not treated as secrets.

## Scope and method

Baseline: commit `320283ef770b6d3a58569c62b02c6153145df722`.

- Non-shallow Git repository; all locally available refs scanned.
- 2302 reachable commits in the initial local inventory, 2301 in origin/main.
  The scanner reported 2000 patch-bearing commits; merge/ref counts are different.
- Standard Gitleaks rules, decoded-content support and a second pass with archive
  inspection to depth two. Redacted output was used for reported results.
- Inventory of 3500 tracked paths, including credential-like filenames, private-key
  markers, credential-bearing URLs, email addresses, public IPs and absolute links.
- The four credential-URL matches were synthetic negative tests, not account logins.
- 52 current PNG files were processed with local OCR; scanning that text produced
  no secret finding. A representative native UI screenshot was visually inspected.
- The two tracked compressed archives were included in the archive-aware pass;
  the delivery archive's member inventory was inspected.
- Provider credentials were not tested against external services. No keys were
  revoked, accounts changed or services restarted.

The scanner release was downloaded from the official Gitleaks GitHub release
and checked against its published SHA-256 manifest. The CI workflow pins both
the scanner archive hash and the checkout action commit.

## Changes and ongoing checks

English public guides and their retained French editions use example network
addresses. Historical documents stay in their original language, with only
identified operational values sanitized. This editing means an old source
manifest describes its original artifact, not the sanitized documentation bytes.

Ignore rules now cover common credential/state files and dependency directories.
They do not affect files already tracked. A CI workflow runs secret scanning
and validates public documentation links and English/French page pairs.

Run from a full checkout with Gitleaks 8.30.1 on PATH:

```sh
bash scripts/check-secrets.sh
python3 scripts/check-docs.py
```

The secret command checks Git history, not uncommitted working files.
Before committing, also inspect the staged changes:

```sh
gitleaks git . --staged --redact=100 --no-banner --ignore-gitleaks-allow
```

## Remaining exposure and limits

**The old operational values remain in older Git commits.** This task does not
rewrite public history, change commit authorship or invalidate existing clones.
Removing old values from every published reference would need a separately
coordinated history rewrite. No active credential was found that would require
immediate rotation based on this audit.

Normal Git author metadata remains. Internal policies, specifications, design
notes and dated reports are still tracked and publicly readable where the
repository is public. They are separated in navigation, not made private.
Their presence is a publication choice, not something `.gitignore` can undo.

The scan covers fetched/local Git refs and current tracked files. It does not
audit GitHub issues, discussions, release attachments, external forks/clones,
unfetched refs, provider accounts or local ignored files. OCR is fallible and
does not replace exhaustive visual review; old image revisions were not OCRed.
Unknown/custom secret formats and sensitive prose can evade automated rules.

Do not use this report as a claim that the entire history was scrubbed or that
the repository has a general security certification.

This report uses the Europe/Paris date. Remaining loopback addresses and
documentation examples are not deployment metadata.

CODEOWNERS requests maintainer review for the scripts, exclusions and workflow.
Enforcement depends on GitHub branch rules, which this task does not change.
CI uses the proposed revision's files; it cannot defend against malicious changes
to its own checks without trusted review.

## Historical notification fixture — 2026-10-25

The full-history delivery check found one additional false positive in commit
`6f2ad5166ec1762b6726fbbf51acaf3ebc79c73f`, in
`crates/bridget-daemon/src/mobile/store.rs`, line946, rule `generic-api-key`.
It is the notification receiver string in the unit test
`mobile177_notification_reassociation_moves_receiver_without_moving_authority`.
The test uses `Store::memory()`, synthetic timestamps and generated local pairing
keys. It tests receiver reassociation without changing device authority. It does
not call an external notification service. The receiver string consists of two
lowercase words separated by underscores and the sequential suffix123456789.
It is a synthetic fixture, not a provider credential. Its value is not repeated
here. The exception records only that immutable commit/path/rule/line fingerprint;
no current path, future commit or scanner rule is excluded.
