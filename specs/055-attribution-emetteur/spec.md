# Session 055 — Attribuer l’émetteur dès `turn_start`

## Métadonnées

- **Statut** : en cours
- **Branche** : `session-055-attribution-emetteur`
- **Base gelée** : `2750bdf6889e1f664fff11a72a4719d00509e599`
- **Objectif** : `20c7a2a8-4bff-49b5-b1ca-1ffe1145ee92`
- **Délégation** : `0d3018c4-e397-46af-a543-61f2e14eaacf`

## Problème mesuré

Les pilotes Codex et Claude écrivent `turn_start` avec le corps seul. Le
renderer Attach applique alors son repli historique `humain` à un émetteur
absent. Une ligne émise par un agent peut donc être affichée comme une ligne
humaine avant que `prompt_dispatched`, enrichi mais postérieur, n’arrive.

## Propriété

Pour le message `jc2 → bridget`, de corps `TRANCHE SPEC 052 POUSSEE`, chacun
des deux pilotes doit écrire `turn_start.payload.from == "jc2"`. La même ligne
JSONL, rendue par Attach, doit contenir `jc2 →` et ne doit pas contenir
`humain →`.

## Rendu des journaux historiques

- Si `prompt_dispatched` fournit un `from` attesté après un `turn_start` qui
  en était dépourvu, Attach remplace l’en-tête initial par cet émetteur.
- Si aucune entrée du tour ne porte de `from`, Attach rend
  `émetteur non attesté` : une provenance inconnue ne devient jamais humaine.
- `ui.rs` et `app.js` appartiennent à un autre correctif.
- Aucun changement de contrat de journal ou de protocole n’est nécessaire.

## Scénarios et preuves

1. Un faux fournisseur Codex complet livre le message par le pilote réel ; le
   témoin lit le JSONL écrit par `JournalWriter` puis rend cette même ligne dans
   Attach.
2. Le scénario jumeau Claude emprunte le même chemin.
3. Retirer `from` de `turn_start` Codex doit tuer le témoin Codex sur
   l’assertion métier de provenance ou d’en-tête, avant toute réponse métier.
4. Retirer `from` de `turn_start` Claude doit tuer le témoin jumeau.
5. Un même rendu permanent rejoue deux tours historiques : l’un enrichi par
   `prompt_dispatched.from=jc2`, l’autre sans provenance. Il exige
   respectivement `jc2 →` et `émetteur non attesté →`, jamais `humain →`.

## Critères d’acceptation

- `from` est persisté dès `turn_start` dans les deux pilotes.
- Les deux témoins de traversée passent.
- Les deux mutants meurent sur une assertion de propriété, puis sont restaurés
  par empreinte et rejeu nominal.
- Les mutants qui ignorent l’enrichissement tardif ou rétablissent le repli
  humain meurent sur ce même témoin de rendu.
- `cargo check --workspace --all-targets` passe.
