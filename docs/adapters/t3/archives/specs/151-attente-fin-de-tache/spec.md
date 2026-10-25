# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/spec.md](../../../../../../specs/151-attente-fin-de-tache/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles »
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`). Critères
visés : Attente 2 vers 5, Livraison 4 vers 5, Suivi 4 vers 5. Source de la
comparaison : `specs/150-socle-delegation/research-t3-bridget.md` (mécanismes
A1, B1, B3, B4). Contre-revue : `adversarial-review-codex.md`.

<a id="extrait-2"></a>

## Extrait 2

Un parent attend trois enfants. Trois résultats arrivent pendant le même
intervalle. Une seule réponse d'attente porte les trois. Hors attente, les
messages `native-result-*` restent regroupés en un seul tour par le pont
(session 114 : 8 messages ou 32 Kio, `t3code.rs:50-51`, `:2073-2092`).

<a id="extrait-3"></a>

## Extrait 3

- **FR-023** : Les tâches terminées au même moment sont rendues ensemble dans une
  seule réponse d'attente (FR-003).
- **FR-024** : Les messages `native-result-*` restent de la famille « message sans
  réponse attendue » du pont T3. Un test fige ce fait.

<a id="extrait-4"></a>

## Extrait 4

- La session 150 est livrée : `reply_timeout`, `delivery_acked`, lecture de phase
  d'une remise (tâche T005 de 150), refus `referent_paused`, rétention.
- Le délai de 600 s par défaut et 3600 s au plus reprend T3 (10 min, 60 min).
- Un hôte peut couper un appel d'outil avant `wait_timeout` (Codex : 60 s par
  défaut selon sa documentation, à vérifier en T001). L'appel suivant reprend
  l'attente ; rien n'est perdu puisque l'attente ne consomme rien.
- Le résultat publié ne change plus (docs/delegation-native.md:43).
- Les tests suivent la recette `umask 077` (quickstart).
- La livraison suit les sessions 147 à 150 : fichiers sur disque, sans relance de service.

<a id="extrait-5"></a>

## Extrait 5

1. **Délai par défaut et délai des hôtes.** Valeur choisie : 600 s par défaut,
   par le contrat natif. Un parent Codex monté par T3, hors du lanceur Bridget, garde le
   délai d'outil de sa configuration (60 s par défaut) et verra l'appel coupé
   avant 600 s ; il rappelle l'attente. Faut-il demander à T3 de transmettre
   `tool_timeout_sec` ? Défaut retenu : documenter, et fixer 3700 s seulement dans
   le montage Bridget.
2. **Garder l'accusé de lecture explicite `bridget_task_ack` (US7) ?** Sans lui, la
   boîte distingue « à remettre », « en vol » et « remis » mais pas « lu ».
   Défaut retenu : oui, en dernière priorité (T024 et T025), supprimable sans
   effet sur le reste.
3. **Accepter le doublon (attente puis message) ?** Défaut retenu : oui. Une
   suppression automatique du message exigerait une preuve de réception que
   l'attente ne peut pas donner (réponse écrite avant d'être reçue, appel annulé
   par l'hôte). Une suppression future devrait reposer sur un accusé explicite.
