# Contre-revue du plan102 — bdget

Date : 2026-09-16. Demande20:58:05CEST, réponse reçue avant21:01:08CEST,
borne annoncée5min. Agent127bccff-8490-453a-8182-884b749ec41e, client Claude/T3.
Fournisseur/modèle déclarés par le relecteur : Claude, Fable5.1
(`claude-fable-5-1`) ; annuaire modèle inconnu, déclaration non attestée séparément.
Relecture indépendante d'un client/fournisseur déclaré différent de Codex/OpenAI.

Demande Bridget : mcp-99715-6aaae6bd-1 ; réponse :
t3-982430dd-54e8-4d86-81d0-b412314e2a9d. Verdict : **APPROVE_WITH_CHANGES**.

## Question et périmètre

Challenger post→wake→ACK, plages de lecture perdues, confidentialité du ledger,
anti-relais DM T3/wrappers et complexité. Cinq artefacts lus : spec, plan,
data-model, contracts/thread-api, reuse-audit. Tasks non encore créées.
Consigne : lecture seule, aucun fichier modifié/commit/test/service/API payante/GPU.
Le relecteur a confirmé n'avoir rien écrit ni lancé.

## Traitement vérifié des objections

| Objection | Vérifiée comment | Retenue | Raison / correction |
|---|---|---|---|
| Issue inconnue bloquant toutes les mentions futures | plan§6 contre condition de départ data-model | Oui avec borne | Après échéance, seule une NOUVELLE mention strictement supérieure autorise nouvelle génération ; jamais retry mêmeborne ; ACK tardif corrélé àancienne |
| ACK au-delà10min oblige à relire sans nécessité | algorithme reçu/base_seq ; absence de risque tant que ligne pas remplacée | Oui | Échéance libère réservation ; ACK tardif reçu encorecourant accepté, remplacé refusé |
| Identifiant/cadence visibles dans reçus généraux | corps alerte et ledger099 | Oui | Limite de confidentialité explicitée dans spec ; aucun corps/titre |
| Show révèle disponibilité des membres | contratshow et annuaire existant | Oui | Limité aux disponibilités/capacités existantes ; pas de curseurs des autres |
| reply implicite peut viser un expéditeur synthétique | wrapper.rs:2239 n'écrit que si reply=true ; cli.rs:3689 relit last-sender | Partiellement | L'alerte reply=false n'écrirait pas directement l'expéditeur, mais l'ancien DM reste une cible dangereuse ; marqueur typé et refus nommé ajoutés |
| Supprimer toute borne de départ et rotation, budget099 suffisant | idempotency/send_delivery consulté ; aucun plafond global d'alertes de fils prouvé | Non pour la borne, oui pour éviterun ordonnanceur neuf | Garder 5/s,16candidats, ordre last_attempt dans maintenance existante ; aucun service/scheduler indépendant ; justifié par coexistence DM |
| OrdreUUID peu ergonomique | contratlist | Oui comme limite | Ordre technique stable déclaré ; pas de tri métier nouveau |
| Accès humain promis sans chemin d'accès | hypothèse spec vs identitérequise | Oui | L'humain lit via son agent membre ; pas de privilège humain API ajouté |

Les sources internes ont été relues pour les objections de code ; les objections
sur le futur algorithme ont été confrontées aux invariants et aux courses du
modèle. Aucune objection n'a déclenché de modification de production.
V16/V25/V27/V29 et les tâches associées portent les nouveaux cas.

## Limites de la revue

Revue d'un plan, pas preuve de fonctionnement. Aucun test102 exécuté ; les tâches
et corrections finales feront l'objet d'Analyze ici et d'une nouvelle recette
après implémentation. L'identité commerciale du modèle est une déclaration du
relecteur, pas une inférence fiable à partir du seul nom du client CLI.
