# Security and privacy

[English](SECURITY.md) · [Français](SECURITY.fr.md) · [Audit](docs/security-audit.md)

## Report a vulnerability privately

Do not put a credential, private transcript, live database or exploitable
account details in a public issue.

Use GitHub's private vulnerability reporting on this repository if the
**Security → Report a vulnerability** action is available. Otherwise ask the
maintainer for a private reporting channel without including the sensitive
payload. This policy does not claim that GitHub private reporting is enabled.

Include affected versions, a minimal reproduction with synthetic data, impact
and the relevant source location. Redact values; do not send a working secret
to demonstrate that it works.

## Repository rules

Keep provider keys, SSH private keys, cookies, credential-bearing URLs, account
profiles, live state/databases and raw conversation exports outside Git.
Use `example.com`, `example.net`, `example.invalid` or documentation IP ranges
for examples. A harmless generic username is not by itself a credential.

The ignore file is a safety net for **untracked** files. It does not remove
files already committed, their history, tags, release assets, forks or clones.

`scripts/check-secrets.sh` runs Gitleaks with redacted output over fetched Git
history. Exact immutable fingerprints in `.gitleaksignore` record reviewed
historical false positives. Do not ignore an entire source directory, all tests,
a whole secret rule or a new unexplained finding.

## If a real secret is found

1. Revoke or rotate it at its issuer first. Do not test a suspected leaked
   credential against the provider.
2. Remove it from the current files and replace the example with a placeholder.
3. Inventory affected refs, release assets and other copies privately.
4. Coordinate any history rewrite separately. It changes commit identities,
   can invalidate signatures and requires collaborators to resynchronize.
5. Rescan, retain a redacted incident record and verify the issuer-side rotation.

Deleting a line or making a repository private does not revoke a credential.
A clean automated scan is not a guarantee of privacy or absence of secrets.

## Runtime boundary

Bridget attests cooperative clients and applies project/permission checks.
It does not isolate a hostile process controlling the same operating-system
account and reading that account's private files. Protect the account and state
directory. SSH federation does not replace provider account security.
