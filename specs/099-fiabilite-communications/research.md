# Recherche 099

## Constats de terrain

Les essais avant correction sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/099-fiabilite-communications/spec.md. Les premiers
échecs du harnais étaient des erreurs d'isolation (socket hors home, HOME enfant
absent) : corrigées dans le harnais après lecture des journaux, jamais présentées
comme des défauts Bridget. Deux scripts aboutissent ensuite avec leurs enfants
arrêtés ; les comptes fournisseurs réels n'ont pas été utilisés.

## Décisions et alternatives

| Sujet | Décision | Alternative écartée et raison | Impact maintenance |
|---|---|---|---|
| Destinataire lent | Écriture bornée existante hors état global | Réécriture async du daemon : périmètre excessif | Une règle d'écriture déjà testée |
| Identité | Preuve privée par incarnation et contrôle de chaque voie d'envoi | UUID secret/issuer_scope/UID seul : ne prouve pas l'agent ; OAuth : nouveau service inutile | Une attestation commune MCP/CLI/Client |
| Réponses | État existant du fil enrichi jusqu'à confirmation | Nouveau broker/outbox SQL : doublon ; supprimer après send : perte constatée | Une machine d'état inspectable |
| Contrôle t3code | File existante et contrôle réactif | Thread bloquant wait_idle : annulation traitée trop tard | Sérialisation et contrôles au même endroit |
| Journal | Contenu complet dans les bornes existantes, lacune explicite sinon | Coupe silencieuse : observation trompeuse | Réutilisation de JournalWriter |

La recherche de l'identité a découvert deux voies à traiter en plus de Register mcp :
CLI éphémère avec from_declared=false, et SendIdempotent dont issuer_scope n'est
qu'une portée de rejeu. Les trois appartiennent au même défaut d'autorisation.

## Références primaires consultées le 2026-09-16

- [OWASP Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html) :
  refus par défaut et contrôle sur chaque accès ; appliqué aux voies Send et SendIdempotent.
- [CISA Secure by Design](https://www.cisa.gov/sites/default/files/2023-06/principles_approaches_for_security-by-design-default_508c.pdf) :
  sécurité dans les valeurs par défaut ; pas de fallback silencieux au mode vulnérable.
- [Practical Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html) :
  tests proches de la responsabilité, intégration ciblée aux frontières.
- [Google Testing Blog, tests hermétiques](https://testing.googleblog.com/2016/10/?hl=no) :
  préférer des dépendances de test locales et maîtrisées. Le faux t3code est déjà fourni.

Baselines utilisateur consultées : security-compliance, testing-quality ;
standards-tests et revue adverse. Pas de choix de dépendance nouvelle.
Pas de scan CVE et pas de revendication de certification de sécurité.
