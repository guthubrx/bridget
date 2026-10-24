# Documentation

[English](README.md) · [Français](README.fr.md) · [Project overview](../README.md)

English is the default language of the public guides. French editions remain
beside them with a `.fr.md` suffix. The guides are organized for use; dated
implementation records remain historical evidence, not installation instructions.

## Public guides

| Topic | English | Français |
| --- | --- | --- |
| Install and register an agent | [Installation](communication-installation.md) | [Installation](communication-installation.fr.md) |
| Send, reply, inspect and manage queues | [Communication](reference-communication.md) | [Communication](reference-communication.fr.md) |
| Delegate, continue and cancel work | [Native delegation](delegation-native.md) | [Délégation native](delegation-native.fr.md) |
| Connect machines | [SSH federation](federation-services.md) | [Fédération SSH](federation-services.fr.md) |
| Use Bridget on Android | [Mobile](mobile.md) | [Mobile](mobile.fr.md) |
| Build and maintain caches | [Build caches](build-cache.md) | [Caches](build-cache.fr.md) |
| Understand compatibility changes | [Deprecations](DEPRECATIONS.md) | [Dépréciations](DEPRECATIONS.fr.md) |
| Published releases | [Changelog](../CHANGELOG.md) | [Journal](../CHANGELOG.fr.md) |
| Report a security issue | [Security](../SECURITY.md) | [Sécurité](../SECURITY.fr.md) |
| Repository privacy checks | [Audit](security-audit.md) | [Audit](security-audit.fr.md) |

## Historical and contributor records

These records retain their original language, primarily French. They may
describe an older protocol, a particular deployment or an unfinished proposal.
A historical “PASS” is not a current fleet health check.

- [Architecture decisions](decisions/README.md).
- [Specifications and dated validation evidence](../specs/README.md).
- [Older operational reports](rapports/README.md).
- [Archived handoff cards](cartes-reprise-28-08/README.md).
- [Agent instructions, in French](../skills/bridget/SKILL.md) and
  [detailed command material, in French](../skills/bridget/references/commandes.md).

Other dated notes at the root of this directory, the design material in
`conception/` and `lentilles/`, and `catalogue-du-du.md` are historical
development records. Use the public guides above for the default reading path.

## Documentation changes

Update the English page and its French counterpart when a public contract
changes. Keep CLI flags, JSON fields, error codes and code identifiers unchanged
during translation. Use example domains and documentation IP ranges instead
of deployment addresses. Never paste credentials, private transcripts or raw
state exports into a guide.
