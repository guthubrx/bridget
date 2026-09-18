# Contre-revue adverse de l'implémentation 103 — cursor-listen

Date : 2026-09-17. Demande 22:39 à cursor-listen (client Cursor, UUID
04c521fe-3282-41e2-bbc7-e1590ce3c351 ; les relecteurs Codex du projet étaient en erreur),
réponse reçue vers 22:45. Lecture seule confirmée par le relecteur ; fournisseur/client
déclaré différent de Claude, identité commerciale du modèle non attestée.

## Question posée

Cinq points : déterminisme du rendu v1 et rejeu, validation vs contrat, sécurité et
inertie, délégation au transport 099 et codes de sortie CLI, minimalisme. Verdict attendu
APPROVE / APPROVE_WITH_CHANGES / BLOCKED avec fichier et fonction.

## Verdict reçu : APPROVE_WITH_CHANGES

« Pas de lecture de source, pas de corps volatil, pas de voie pour faire passer un dossier
reçu comme attesté par Bridget. »

| Objection | Vérifiée comment | Retenue | Raison / action |
|---|---|---|---|
| LOW — `is_plain_http_url` refuse `HTTP://…` (casse du schéma) | lecture du code : comparaison de préfixe sensible à la casse ; RFC 3986 rend le schéma insensible | Oui | Schéma comparé en ASCII minuscule, URL conservée telle quelle ; test S08 complété (`HTTPS://Example.org/X`). |
| LOW — schéma MCP plus permissif que le runtime (`minLength:1` laisse `" "`, bornes en caractères) | le runtime refuse déjà ces cas (tests S03/S05) | Oui (documentation) | Description de l'outil : « le schéma est indicatif : le validateur fait autorité ». Le schéma reste un filet, pas l'autorité. |
| NOTE — clé 099 volatile si `id`/`issued_at` omis au premier envoi | contrat et recette : préparer la clé avant le premier appel si la perte du reçu doit être récupérable | Non (déjà couvert) | Aucune modification ; la skill et la référence insistent sur la capture du reçu. |
| Cosmétique — l'aide CLI redit `WARNING_DETAILS` | choix voulu : sortie humaine autoportante | Non | Pas de logique dupliquée ; conservé. |

Aucune régénération de fichier ; deux corrections ciblées. Recette complète relancée après
ces corrections (voir implementation.md, T020).
