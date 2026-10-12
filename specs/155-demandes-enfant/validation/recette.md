# Recette155 — quatre cas PASS, provenances distinctes

Checkpoint courant R2 : In Progress, 42/42 acquises, T040 validée sur le reçu
réel antérieur au correctif cwd. Ce correctif a maintenant 71 PASS ciblés sur le code courant. Converge P3 est historique ; relecture du delta après
audit attendue. Les anciens statuts et PASS ci-dessous gardent leur snapshot.
État courant : implémentation réalisée à 100 %, 42/42 tâches cochées par le
principal, T040 validée après accord explicite de l’utilisateur et quatre cas
PASS. Le pipeline reste In Progress : Converge P3, audit final, revue code R2
et livraison restent à faire. Aucun GREEN workspace global n’est revendiqué.
Les checkpoints 41/42, Human PENDING et corrections en cours ci-dessous sont
historiques et supersédés par les preuves finales ajoutées en fin de document.
État historique après le cycle C1 : In Progress, 41/42 ; T040 seule ouverte et
validation explicite de l’utilisateur attendue. Corrections H1/H2/H3 en cours.
Aucune résolution n’est attestée avant les preuves ciblées. Converge P2 reste
un checkpoint historique avant ces correctifs ; P3 reste à faire. Les résultats
antérieurs décrivent leurs snapshots, sans GREEN global courant.
État : In Progress, 41/42. T041 documentaire est acquise. Le principal a validé
et coché T039 sur les preuves cumulées. Seule T040 reste ouverte. Ce reçu distingue quatre cas réels ; il
ne revendique ni 4/4, ni une recette terminée, ni un GREEN workspace global.
Le rédacteur n’a exécuté aucun test, service ou modèle et n’a modifié aucune DB.

| Cas | Résultat | Fait observé |
|---|---|---|
| Question au parent | PASS historique avant C1 | Réponse accept avec couleur Bleu, remise delivered ; résultat enfant corrélé |
| Expiration réelle 600 s | PASS historique avant C1 | deadline−created=600, état expired, aucun accord ; delivery void |
| Limite workspaceWrite | PASS historique avant C1 | forbidden/refused:command_unbounded, decline delivered, aucun item humain |
| Approbation Human | PENDING | Demande et boîte open ; aucun accord accepté ; validation utilisateur attendue |

## Question au parent

Reçu durable extrait par le principal en lecture seule :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/question155-runtime.json.
SHA256 : eb7813cbed00381f32df1450a02f85ae78279f86743252f931c87dddcfd72bd1.
Cette preuve précède le correctif environmentId local ; la route user_input
est inchangée. La lecture indépendante ci-dessous confirme les mêmes faits.

Lecture SQLite URI mode=ro du 2026-10-12T02:15:27.767822+00:00 :
/private/tmp/b155.2s2_gkz3/ns/bridget.db.
Tâche 8de7fceb-855b-4788-acd6-04ec8a367e83 ; demande
b3dddb7f6ab0951725df7de723736a00. La ligne native_child_requests porte
verdict parent, state answered, outcome answered_by_parent, decision accept,
answer_json={"couleur155":["Bleu"]} et delivery_state delivered.
Le payload de native_delegations est result_available, error=null et contient
« Mission recette155 terminée. Couleur reçue : Bleu. » ; cleanup_done=true.
Cette lecture porte sur la DB synthétique privée. Elle ne prétend pas que
son hash est immuable et ne modifie aucun état.

## Expiration réelle 600 s

Preuve : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/expiry155-r3-runtime.json.
SHA256 : 39fe000fbc5e06f90951947d220a23119fe5aa81b1459e895e184f76388a76ee.
Tâche 9f3efac0-f57e-42d1-869c-cc31bf978bd6 ; demande
a5331f20ad1a983c7726fe9879cc6f6b. created_at=1791769825 et
deadline_at=1791770425 ; le reçu atteste real_wait_seconds=600.
La demande Parent finit expired/outcome expired, decision=null, delivery_state
void, void_reason=closed_before_apply, sent_at=null et delivered_at=null.
Aucun accord n’a été envoyé. Les événements du reçu sont vides ; aucun événement
fournisseur supplémentaire n’est inventé. Le reçu atteste database_read_only=true.

## Refus workspaceWrite et attente Human

Preuve commune capturée à 2026-10-12T02:12:50.982450+00:00 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette155-final-runtime-open.json.
SHA256 : 8dee7fc2f7776e5ebb45dec7aa14d1917f093fd14500dade240e92e69e535dcd.
Namespace : /private/tmp/b155.2s2_gkz3/final-ns.
Binaire SHA256 : fb48b0420c8e305ef4cff9621f67a5fa84c85ac6d78da62deb3e0581d1b3d2ec.

WorkspaceWrite : tâche 8512b027-5afc-4596-bfb9-a8aa5b6ecf7d ; demande
7e0ea0ce245919d60b52e93632199e2d. Les faits indiquent
parent_sandbox=workspaceWrite, duration=once, network=false et chemins non
vérifiés. Verdict forbidden, state refused, outcome refused:command_unbounded,
decision decline, delivery_state delivered et inbox_item_id=null. Le reçu
porte wsw_pass=true. Le refus conserve la limite des droits du parent.

Human : tâche 37356718-0f26-4f4b-b23f-bbc703d5af32 ; demande
7f4868101610d6bd45b22cf7836336ab ; item
eeeccded-e8bb-45e4-9d85-b7a526eaa115. Les faits indiquent
parent_sandbox=dangerFullAccess, duration=once et la commande inoffensive
/opt/homebrew/bin/zsh -lc 'printf RECETTE155_HUMAN'. Dans la capture historique, la demande et la boîte
sont open, decision=null, delivery_state=none, inbox_settled=0.
La tentative Respond du parent est refusée avec parent_cannot_approve.
Le reçu porte human_acceptance_pending=true. La validation explicite de
l’utilisateur reste attendue ; aucune acceptation ni exécution approuvée
n’est attestée. Les trois autres PASS ne valent pas une décision humaine.

Checkpoint historique après T041 : In Progress, 41/42, T039 et T041 documentaire
acquises ; T040 seule ouverte. Les checkpoints historiques 40/41 restent
historiques et sont remplacés par cet état courant. T041 corrige uniquement
le paragraphe Journal du contrat, sans changement de code ou de tests.
Preuve et lectures ciblées :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/implementation.md,
section T041 (transport codex_app_server.rs:165–178,4266–4310 ;
table delegation.rs:307–327).

## Checkpoint historique C1 — avant preuves finales

Le cycle C1 en lecture seule est clos : 0 CRITICAL, 3 HIGH, 1 MEDIUM, 2 LOW.
Archive originale conservée :
/Users/moi/.cache/bridget-delivery155.um4xccv4/audit155-cycle1-original.
Agrégat : /Users/moi/.cache/bridget-delivery155.um4xccv4/audit155-cycle1-original/cycle-1/aggregated-findings.json.
SHA256 : c831e7a10f706e5ef23b10125643ec9938a7b7abb63772010a09333eef5384ab.

Les trois HIGH sont confirmés par lecture statique, sans reproduction runtime :
H1/SEC-001 (faits Human transformés ou raccourcis), H2/REL-001 (écriture STDIO
privée non bornée sous les tables), H3/PERF-081 (notification externe Human
retenant le verrou DaemonState). Leurs correctifs sont en cours. Aucun nouveau
PASS, hash final de source, recette réussie ou finding résolu n’est attribué
à ces correctifs dans ce checkpoint.

La correction du signalement Claude MEDIUM sur emit a été écrite et relue par
le principal : l’erreur ne produit unknown que si le permis d’émission avait
été acquis. Sans permis, aucun octet ne peut avoir été écrit et sent reste
suivi, avec FR027 refait avant un prochain envoi. Aucun champ SQL, marqueur
persistant ou helper nouveau. Les tests ciblés de cette correction restent
attendus. PERF-041 corrige seulement l’annotation de coût en q², sans modifier
l’algorithme. PERF-082 et DUP-001 restent des limites acceptées : budget du tick
renouvelé par émission et duplication locale de reconnexion, sans incident
runtime mesuré ni refactor de confort.

Converge P2 est historique, avant les corrections C1 :
/Users/moi/.cache/bridget-delivery155.um4xccv4/converge155-p2.json.
SHA256 : 9c77dc98c3f40a3f9da51d4a29f703aea0f6aabd962419a61f0914e0c092ecb0.
P3 reste à faire après validation des changements. Le statut reste In Progress,
41/42, avec T040 seule ouverte ; aucune case n’est modifiée par le rédacteur.

La suite workspace unique conserve 2259 PASS / 15 FAIL / 72 ignored, avec
180/180 baseline PASS. Ses 8 échecs historiques et 7 nouveaux sont distincts ;
les 7 nouveaux ont ensuite leurs replays PASS sur le patch antérieur à C1.
Les 71 PASS155 antérieurs ne valident pas les nouveaux correctifs C1. Aucune
seconde suite complète, aucun total final GREEN ou test supplémentaire n’est
inventé. Les reçus de question, expiration et workspaceWrite restent historiques ;
la demande Human attend toujours la validation explicite de l’utilisateur.

## Checkpoint historique avant Converge P3 — 42/42

Le principal a validé et coché T040 après l’accord explicite « Oui, accepter et
valider si les quatre cas réussissent ». Le reçu conserve cet accord à
2026-10-12T03:02:02.400752+00:00. Les quatre cas sont PASS. L’implémentation
est réalisée à 100 %, 42/42 ; le pipeline reste In Progress. Converge P2 est
historique avant les correctifs C1 ; P3, audit final, revue R2 et livraison
restent à faire.

Reçu actuel :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette155-postfix-final.json.
SHA256 : 153b70cbbbaa94315b87f192a8fa67fc217f607eb1cc70c5e9ea5b047b6cc9f3.
H1/H2/H3 et le permis d’émission MEDIUM ont leurs correctifs et preuves ciblées
fraîches. PERF-041 change seulement le commentaire de coût. PERF-082 et DUP-001
restent les deux LOW acceptés dans le périmètre. Cela ne vaut pas audit final.
La suite complète unique reste 2259 PASS / 15 FAIL / 72 ignored avant correctifs,
avec 180/180 baseline PASS et les sept nouveaux échecs ensuite rejoués PASS.
Aucune seconde suite complète n’a été lancée.

Tests ciblés frais et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/tests-155.md,
section Tests ciblés frais après les correctifs C1.

## Recette finale après correctifs — quatre cas PASS, provenances distinctes

| Cas | Résultat | Provenance |
|---|---|---|
| Question au parent | PASS | Capture réelle antérieure question155-runtime.json réutilisée |
| Expiration 600 s | PASS | Capture réelle antérieure expiry155-r3-runtime.json réutilisée |
| Limite workspaceWrite | PASS | Capture réelle antérieure recette155-final-runtime-open.json réutilisée |
| Human accept | PASS | Nouvelle recette après correctifs, consentement explicite utilisateur |

Les trois premières preuves gardent leur ancien binaire. Le reçu explique que
leurs chemins nominaux restent inchangés et que les scénarios touchés passent
sur les sources finales. Elles ne sont pas présentées comme une exécution du
binaire final. Aucun second délai réel de 600 s ni seconde suite complète ne sont
inventés. Les empreintes des trois entrées sont conservées dans input_sha256
du reçu final. La nouvelle observation Human porte sur le namespace isolé :
/private/tmp/b155.2s2_gkz3/postfix-ns.
Binaire SHA256 : 4daf5452d3e475f7d89fd62bcfec980cbaf3e0ab223f2952a37703807a7bfe5c.

Tâche Human fraîche : 466ef490-6eb5-43eb-b468-245e1ab9b748.
Demande : 041ee2ce834ccc2bfb8ebc1efb35a70d.
Item : b4474ff8-4f18-4f1a-8530-060991aa92bc.
Avant l’accord humain, le parent reçoit parent_cannot_approve par MCP id5.
Après l’accord utilisateur, la CLI humaine accepte cette seule demande.
L’événement fournisseur atteste l’unique commande
/opt/homebrew/bin/zsh -lc 'printf RECETTE155_HUMAN', sortie 0 et résultat
RECETTE155_HUMAN. La ligne finit answered_by_human, decision=accept,
decided_by=human, delivery_state=delivered ; inbox_settled=1. L’item est
resolved et ACK à 1791774509. La tâche est result_available avec error=null.
L’ancienne tâche 37356718-0f26-4f4b-b23f-bbc703d5af32 est annulée et n’a
jamais été acceptée ; son attente historique est supersédée par ce cas frais.

Note de provenance du reçu : la version canonique porte user_validation.executed=true
après l’accord et la commande exécutée. Le snapshot avant acceptation, qui portait
executed=false, reste conservé séparément :
/Users/moi/.cache/bridget-delivery155.um4xccv4/recette155-postfix-final-before-state-note.json.
SHA256 historique : 821f84d7b54cea295688daa99b1ade7aea81f425af52e20940fbdfec8b686c3b.
La version canonique garde user_validation_before_accept. Cette précision ne
change ni les quatre cas, ni l’événement CLI, ni les sources ou tests.

## Checkpoint R2 — correctif cwd prouvé par les ciblés

État courant : In Progress, 42/42 tâches acquises, T040 validée. L’objection
MEDIUM cwd de la revue R2 est retenue ; le correctif minimal a maintenant 71 PASS ciblés, RC0, sources 214 stables.
Le reçu post-cwd ci-dessous remplace l’attente des tests ; aucun hash futur
de source ou résultat global n’est inventé. Converge P3 est un
checkpoint historique antérieur au correctif cwd (03:23 selon le principal).
La vérification du delta après audit reste attendue. Aucun audit final,
livraison ou GREEN workspace global n’est déduit.

La recette réelle canonique reste immuable :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette155-postfix-final.json.
SHA256 : 153b70cbbbaa94315b87f192a8fa67fc217f607eb1cc70c5e9ea5b047b6cc9f3.
Elle atteste quatre cas PASS et executed=true. Son Human frais était réalisé
après H1/H2/H3, mais avant le correctif cwd. Les trois autres cas gardent leurs
captures et ancien binaire. Ces quatre cas ne sont pas déclarés sur le dernier
binaire incluant cwd. Le delta cwd est validé uniquement par les ciblés avec
fournisseur synthétique lorsqu’ils sont reçus ; aucune nouvelle recette de
modèle, attente réelle de 600 s, suite complète ou permission utilisateur n’est
exigée ou annoncée dans ce passage documentaire.

Converge P3 historique :
/Users/moi/.cache/bridget-delivery155.um4xccv4/converge155-p3.json.
SHA256 : 74a0df9585f1d32b0811c54ed289a6e5d857370b1eaf65a195827d6ad9dce129.
Les 71 PASS C1 et quatre régressions distinctes conservent leur propre snapshot
avant cwd. La suite unique 2259/15/72 n’est pas rejouée ni transformée en GREEN.

La section quatre cas PASS ci-dessus reste une recette réelle avant cwd.
Le reçu canonique n’est pas réécrit et l’ancienne Human annulée n’est pas
réactivée. Aucun nouvel accord humain ou nouvelle commande réelle n’est
sollicité pour ce delta documentaire et synthétique.

Le delta présenté désormais à l’humain ajoute facts.cwd cité entier par Debug
ou null et facts.cwd_verified=false. Commande et cwd ont chacune une borne
1024 octets UTF-8 ; type invalide ou longueur hors borne donnent details_incomplete.
Le budget facts 4096 reste inchangé, sans résolution ni fallback task.cwd.
Cette nouvelle projection n’est pas attribuée au reçu réel antérieur.

## Preuve ciblée post-cwd — 71 PASS courants

native155-cwd-r1 : RC0, 64,81 s, 71 PASS / 0 FAIL (63 daemon, 1 E2E,
7 transport). Les 214 empreintes avant/après sont strictement égales.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/native155-cwd-r1.json.
Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/native155-cwd-r1.log.
SHA256 du log : 2be93808d3211bfe126877fec072bae9eba5c2d8652f575ac3c353a531d7ba16.
Le correctif cwd a maintenant sa preuve synthétique sur le code courant.
La capture et la relecture du delta après P3 appartiennent au principal.

Les quatre régressions canal087/STDIO restent des preuves distinctes avant cwd ;
aucun total de 75 PASS sur le dernier snapshot n’est revendiqué. La recette réelle
canonique 153b70 reste validée sur ses provenances antérieures à cwd, avec Human
après H1/H2/H3. Aucun nouveau runtime de modèle, second délai réel de 600 s ou second
FULL n’est lancé ou déduit. Statut In Progress, 42/42 et T040 acquise ; audit
final, relecture du delta, revue finale et livraison restent distincts.
