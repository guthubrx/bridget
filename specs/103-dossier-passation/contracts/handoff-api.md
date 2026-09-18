# Contrat 103 — MCP et CLI

## MCP bridget_handoff

Objet strict. action vaut preview ou send et est obligatoire.
draft obligatoire : objective (1..1 024 octets UTF-8 après contrôle non-blanc),
summary (1..8 192), results≤12, decisions≤12, questions≤12, references≤16, limitations≤12,
next_step facultatif≤2 048. Chaque chaîne d'une liste≤2 048 ; result.text≤2 048,
result.evidence facultatif≤2 048. Chaque référence sérialisée≤2 048, label≤256.
Les champs obligatoires doivent être non blancs ; conserver leurs octets d'origine.
Listes vides acceptées. Chaînes facultatives vides refusées (omettre le champ).
Dossier final préfixé≤16 384 octets ; ces maxima par champ ne s'additionnent pas en un droit
à dépasser le total. Aucun nombre à virgule pour séquences.

Transport send : to UUID obligatoire ; reply bool facultatif=false ; reply_timeout entier
positif seulement avec reply=true (mêmes bornes que send) ; in_reply_to facultatif ;
id et issued_at obligatoirement ensemble s'ils sont fournis, mêmes règles que send.
preview refuse tous ces champs de transport, pour rendre son absence d'envoi incontestable.
Les champs inconnus, null explicites, faux types et version fournie sont invalid_params.

Corps EXACT : marqueur "[Bridget handoff v1]\n" puis JSON pretty (indentation 2 espaces)
de la structure ordonnée data-model.md, sans saut de ligne final.
Échappement v1 figé : guillemet et antislash échappés ; contrôlesJSON avec les formes
courtes \b, \f, \n, \r, \t quand applicables, autresU+0000..001F en \u00xx minuscule ;
/ non échappé, caractèresUnicode hors contrôles conservés en UTF-8. Pas d'espaces finaux.
Fichiers dorés de tests pour Unicode, /, contrôles, listes vides et ordre des champs ;
une mise à jour serde_json ne doit pas modifier ces octets. Si nécessaire maintenir
ce renduv1, pas changer le corps sous une clé déjà envoyée.
Les tableaux conservent l'ordre. Ne jamais inclure une date
courante, un nom annuaire ou une valeur d'instance calculée dans ce corps.
Les valeurs par défaut sérialisées assurent le même rendu entre champ absent et liste vide.

Exemple preview :
```json
{"action":"preview","draft":{"objective":"Corriger la pagination","summary":"Le défaut est reproduit sur la deuxième page.","results":[{"text":"Le test de chevauchement échoue.","evidence":"Exécution déclarée par A sur la fixture synthétique."}],"questions":["La purge concurrente est-elle responsable ?"],"next_step":"Relancer le test ciblé et observer les IDs."}}
```

Sortie preview (exemple schématique : le champ body contient réellement le corps complet) :
status=preview_valid, body:string, bytes:integer, warnings:string[].
Warnings fixes : sources_not_verified ; retention_follows_ledger ;
ledger_visibility_not_recipient_private. Les textes français détaillent ces limites.
Aucun ID/date d'envoi n'est fabriqué en preview.

Exemple send minimal :
```json
{"action":"send","to":"11111111-1111-4111-8111-111111111111","id":"handoff-pagination-01","issued_at":1789588800,"draft":{"objective":"Corriger la pagination","summary":"Défaut reproduit, cause encore incertaine."}}
```
Horodatage d'exemple à remplacer par l'instant réel avant le PREMIER envoi.
Sortie send = reçu execute_send existant + handoff_version=1, bytes et warnings constants.
Ne pas renvoyer body. L'appelant garde l'entrée originale pour le rejeu.
Pas de nouveaux états succès ; isError et refus 099 restent inchangés.

## CLI

bridget handoff preview --json-stdin [--json]
bridget handoff send --json-stdin [--json]

stdin contient le même objet que le MCP, y compris action ; il doit correspondre à la
sous-commande. Ne pas proposer --file ou lire des chemins référencés. Borne stdin64Kio.
--json produit le même objet de résultat que MCP ; sortie humaine preview imprime le corps
et les limites, send imprime ID/date/statut et un avertissement compact.
Code sortie : 0 preview_valid/accepted ; 2 paramètres invalides ; 1 erreur, refus ou issue
non confirmée (in_flight/outcome_unknown) avec reçu toujours imprimé. Ne pas interpréter
le code1 comme une autorisation de nouvel envoi : rejouer la même clé si demandé.

Identité : send réservé au contexte d'agent attesté dans cette première version. Pas de
--from ni de posture humaine auxiliaire nouvelle. preview local ne nécessite pas de daemon.

## Invariants de sécurité et non-effets

Valider tous les champs avant execute_send. Aucun accès fs/HTTP/source depuis la validation.
Pas d'appel à bridget_events, pas de notification @all, pas de clôture de demande autre que
le mécanisme in_reply_to déjà existant, pas de réponse requise par défaut.
DND/offline suivent send, pas de dérogation. Aucune mutation d'artefact.
