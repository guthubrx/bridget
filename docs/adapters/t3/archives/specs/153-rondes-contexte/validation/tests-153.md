# Archives techniques de l’adaptateur T3

Document d’origine : [specs/153-rondes-contexte/validation/tests-153.md](../../../../../../../specs/153-rondes-contexte/validation/tests-153.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Compteur courant **7/29**. Aucun total global recomposé ni succès de modèle,
T3 ou production153. Le masque077 reste obligatoire pour les prochaines
commandes prévues ; aucune reprise longue de baseline n'est requise ici.

<a id="extrait-2"></a>

## Extrait 2

Après preuves ROOT, relecture et autorisation explicite : seules T008,
T009, T010 et T011 sont cochées. Compteur courant **11/29**.
T012/T029 restent ouvertes ; aucun total global ni succès de modèle/T3/
production déduit. Les preuves antérieures restent intactes.

<a id="extrait-3"></a>

## Extrait 3

Seule T012 nouvellement cochée après preuve, relecture et GO du principal :
**12/29**. T013/T014/T016/T018/T028/T029 restent ouvertes. T022/T023
attendent T018 ; aucun succès global, modèle, T3 ou production déduit.

<a id="extrait-4"></a>

## Extrait 4

Self-reviews façade/stockage et code relus :
- T016 : Task.result_message_id est utilisé par la vue courante201,
  authorize_retained_result1340, l'envoi tick et copy_delivery_acks2662 dans
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs.
  Le test resultat_id_distinct_par_ronde reçoit deux vrais DeliverIdempotent,
  ID historique de R1 puis suffixe-r2, et recopie les vrais ACK courants.
- T018 : task_view147 ajoute ronde/délai/continuable en lecture pure.
  L'oracle appelle directement current_task_view (vue commune de Status),
  avec payload durable byte-identique. TaskWait utilise le handler public.
  Aucune trame Status envoyée n'est revendiquée pour cette assertion.
- T020 : vrai change_receiver du watch et vraie lineage_page. Après Continue,
  seq/notification puis mission_pending/working, une tâche/même enfant/forme149.
  Cancel public et arrêt confirmé gardent statut/résultat, sans avis ni seq neuf.
  Le premier résultat provient d'une fixture cohérente après ACK et exécution completed.
- T028 : capture_reply1442 lit la mission passée, compare les identités vivantes
  et owner ; aucun recover_child ni sauvegarde sur cette branche. Les quatre
  chemins D passent, dont le tiers authentifié non absorbé. Preuve locale du
  daemon, pas interface T3 ni modèle réel.
- T029 : copy_delivery_acks2662 lit les anciennes rondes même quand la courante
  est accusée, ne retire que les phases acked et sauvegarde avant purge. Le cas
  accuse_tardif_de_n_autorise_la_purge passe. tree_settled35 exige unacked vide
  hors du OR et delete_tree243 supprime les missions passées dans sa transaction :
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs.

<a id="extrait-5"></a>

## Extrait 5

Compteur courant **20/29**. T017/T019/T021/T025 attendent le seul workspace
final ; T022/T023/T024/T026/T027 restent ouvertes à ce checkpoint.
Aucun E2E, modèle réel, UI T3, production ou GREEN global déduit.

<a id="extrait-6"></a>

## Extrait 6

**T024 : non vérifié.** La recette réelle de mémoire du quickstart n'a pas été
lancée, par décision explicite du principal. Aucun appel de modèle/Claude CLI,
aucune inspection de son authentification, aucun service relancé. Ne pas attribuer
ce non-lancement à une panne d'authentification. La mémoire d'un vrai modèle et
l'interface T3 ne sont pas validées. Le fournisseur synthétique prévu par T019
ne peut pas établir ce souvenir. T024 est cochée pour cette consignation autorisée ;
elle ne transforme pas la mémoire réelle SC-002 en résultat automatisé PASS.

<a id="extrait-7"></a>

## Extrait 7

T021 : V2 confirme **195/195 noms et cibles aux mêmes résultats** que T001,
soit194 PASS/1 ignoré, aucune divergence. L'analyse V1 limitée à libdaemon est
supersédée par V2 incluant les intégrations ; ce changement d'analyse ne rejoue
aucun test. T021 reste ouverte jusqu'à la relecture d'impact des dernières sources.
La comparaison causale du principal confirme les huit échecs historiques151/152 :
un --from non attesté, trois identités auxiliaires absentes, trois refus
auxiliary_credential_required et le HTTP404 T3. Leurs sources d'intégration restent
inchangées. Aucun nouveau échec153 n'est classé hérité.

<a id="extrait-8"></a>

## Extrait 8

T025 exécutée :workspace R1 reste2145 PASS/10 FAIL/72 ignorés RC101 avant deltas
finaux. Huit échecs préexistants visibles ; deux nouveaux et FR-019 résolus par
les seuls contrôles affectés. Aucun GREEN global ni workspace sur hash final.
T026 : self-reviews/couplage/erreurs et diff dans implementation.md.
T027 :29/29, statut Implemented et checklist raccordés. Mémoire réelle, UI T3,
activation, couverture/mutation/benchmark/délai mural60s restent non vérifiés.
