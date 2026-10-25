# Recherche 134 — Noms humains dans les messages Bridget

## Décision 1 — Réutiliser le libellé central

**Décision** : enrichir `BridgetMessage::sender_label()`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/research.md#extrait-1).

**Alternatives étudiées** : modifier chaque transport ; enrichir le corps du
message ; remplacer l’UUID par le nom. Elles dupliquent la règle, mélangent les
données ou rendent le diagnostic ambigu.

**Impact mainteneur** : une seule fonction explique le format visible.

## Décision 2 — Réparer au moment de l’enregistrement

**Décision** : rendre `ensure_agent_ids()` idempotent sur l’identité, le profil
et l’état d’application.

**Raison** : ce chemin est déjà appelé lors de l’enregistrement. Il possède une
transaction et connaît l’identité exacte. Il évite une migration globale de la
base de production.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/research.md#extrait-2).

**Impact mainteneur** : l’invariant est local et vérifiable par un test SQLite.

## Décision 3 — Garder l’UUID visible

**Décision** : utiliser `nom (UUID)`, jamais le nom seul.

**Raison** : un nom peut changer ou entrer en conflit. L’UUID permet une réponse
et un diagnostic sans ambiguïté.

**Alternatives étudiées** : nom seul ; UUID raccourci ; table de correspondance
dans le prompt. Elles retirent une preuve utile ou ajoutent un second mécanisme.

**Impact mainteneur** : aucun changement des règles de routage.
