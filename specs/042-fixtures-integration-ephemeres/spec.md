# Spécification 042 — Fixtures d'intégration éphémères

**Branche** : `session-042-fixtures-integration-ephemeres`
**Base** : `90802b0377741b509f3743c5675544315b6f0f29`
**Dépendances** : aucune

## Contexte

Le 27 août, six répertoires de fixtures d'intégration abandonnés occupaient
6,3 Gio, dont un atteignait environ 1,2 Gio. Le banc de couture de l'identité
de build crée un répertoire temporaire contenant un répertoire de compilation
privé. Il le retire uniquement à la dernière ligne : une erreur ou une panique
conserve donc la fixture entière pour les exécutions suivantes.

L'inventaire sur la base isole ce banc comme le producteur des grosses
fixtures. Son daemon enfant est déjà détenu par une garde qui l'arrête à la
sortie ; la racine de fixture ne l'est pas. Les répertoires de diagnostic
conservés volontairement ne font pas partie du périmètre.

## Scénarios utilisateur et tests

### US1 — Exécuter une couture sans résidu (P1)

Un mainteneur exécute le banc de couture. À sa fin normale, son répertoire de
fixture, y compris les artefacts de compilation, a disparu.

**Critères d'acceptation** :

1. Chaque racine de fixture du banc est distincte, même si deux exécutions
   partagent un identifiant de processus.
2. L'arrêt normal du banc ne laisse aucune racine de fixture.

### US2 — Échouer sans résidu ni processus (P1)

Lorsqu'une assertion ou une panique interrompt le banc, la garde libère la
racine de fixture. Le daemon enfant reste arrêté par sa garde existante.

**Critères d'acceptation** :

1. Une panique volontaire, capturée par le témoin, laisse la racine absente.
2. Le témoin ne compile pas de binaire réel : il vérifie la garantie de
   nettoyage sans produire lui-même un artefact lourd.

## Exigences fonctionnelles

- FR-001 : toute racine créée par le banc lourd est possédée par une garde
  libérée lors du déroulement normal, d'un retour anticipé ou d'une panique.
- FR-002 : l'identifiant de racine comporte une source d'unicité indépendante
  de l'identifiant de processus.
- FR-003 : la garde de répertoire ne masque jamais l'arrêt du daemon détenu
  par la garde de processus existante.
- FR-004 : aucune sortie conservée volontairement pour un diagnostic n'est
  supprimée par cette modification.

## Hors périmètre

- aucun ramassage automatique de `/tmp` ;
- aucune suppression à distance ;
- aucun changement de comportement de production ;
- aucune modification des fixtures de diagnostic explicitement conservées.

## Critères de réussite

- Le témoin de panique constate une racine absente après le déroulement forcé.
- Le témoin d'unicité constate deux racines différentes pour une même étiquette.
- Le banc lourd conserve son oracle fonctionnel existant.

## Hypothèses

- Le répertoire construit par le banc est exclusivement sa propriété.
- La garde de daemon existante demeure l'autorité de terminaison du processus
  enfant ; la nouvelle garde ne tue aucun processus.
