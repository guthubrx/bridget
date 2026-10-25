# Archives techniques de l’adaptateur T3

Document d’origine : [specs/099-fiabilite-communications/research.md](../../../../../../specs/099-fiabilite-communications/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Sujet | Décision | Alternative écartée et raison | Impact maintenance |
|---|---|---|---|
| Destinataire lent | Écriture bornée existante hors état global | Réécriture async du daemon : périmètre excessif | Une règle d'écriture déjà testée |
| Identité | Preuve privée par incarnation et contrôle de chaque voie d'envoi | UUID secret/issuer_scope/UID seul : ne prouve pas l'agent ; OAuth : nouveau service inutile | Une attestation commune MCP/CLI/Client |
| Réponses | État existant du fil enrichi jusqu'à confirmation | Nouveau broker/outbox SQL : doublon ; supprimer après send : perte constatée | Une machine d'état inspectable |
| Contrôle t3code | File existante et contrôle réactif | Thread bloquant wait_idle : annulation traitée trop tard | Sérialisation et contrôles au même endroit |
| Journal | Contenu complet dans les bornes existantes, lacune explicite sinon | Coupe silencieuse : observation trompeuse | Réutilisation de JournalWriter |

<a id="extrait-2"></a>

## Extrait 2

- [OWASP Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html) :
  refus par défaut et contrôle sur chaque accès ; appliqué aux voies Send et SendIdempotent.
- [CISA Secure by Design](https://www.cisa.gov/sites/default/files/2023-06/principles_approaches_for_security-by-design-default_508c.pdf) :
  sécurité dans les valeurs par défaut ; pas de fallback silencieux au mode vulnérable.
- [Practical Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html) :
  tests proches de la responsabilité, intégration ciblée aux frontières.
- [Google Testing Blog, tests hermétiques](https://testing.googleblog.com/2016/10/?hl=no) :
  préférer des dépendances de test locales et maîtrisées. Le faux t3code est déjà fourni.
