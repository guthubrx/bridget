# FR016 — patch production proposé, cache seulement

Source de base : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/crates/bridget-daemon/src/daemon.rs.
SHA256 : 0dfd46715aa3b1440f85b046c791e05f750753cbc26ad864c334d248a1ffedbd. Source WT inchangée à la fin de la préparation. Aucun Cargo, test, build, Git write ou service exécuté.

Patch : /Users/moi/.cache/bridget-delivery156.iuy6stuo/convergence-fix/daemon-fr016-production.patch.
Copie candidate : /Users/moi/.cache/bridget-delivery156.iuy6stuo/convergence-fix/daemon-fr016-candidate.rs.
Manifest : /Users/moi/.cache/bridget-delivery156.iuy6stuo/convergence-fix/daemon-fr016-manifest.json.
ROOT seul applique après son RED. Compilation et comportement restent à prouver par ROOT.

## Recherche et réemploi avant création

Recherche nom/responsabilité par rg dans daemon.rs et specs/156-guidage-file/reuse-audit.md : record_guidance_refusal absent ; record_guidance_outcome à9152 ne traite que des faits déjà disponibles ; check_guidance à9190 trace les refus de authorize mais pas le refus avant cette garde. Le store record_guidance_event, guidance_source et guidance_event_labels sont réutilisés. Aucun store, type, table ou service supplémentaire.

Créer une seule fonction privée record_guidance_refusal est justifié par quatre puits indépendants : prepare_dispatch du SendIdempotent9721, prepare_dispatch du Send historique18876, refus projet Control10167 et refus projet Cancel19517. Son cinquième appel couvre l'échec de lecture des faits dans check_guidance. Cette note fournit le delta de justification à reporter par ROOT dans le reuse-audit existant ; aucun document WT n'est écrit ici.

Le helper prend une action fermée optionnelle, la connexion, la cible et l'ID original. Il ne lit pas guidance_facts, les taux ou les projets. Il obtient seulement l'acteur réel avec guidance_source. Il réutilise les labels identiques, écrit refused:<code>, normalise la cible et date l'événement. Sans action Steer/Interrupt, il ne fait rien. Le contrôle interne conn=None ne trace rien. Un échec de journalisation produit le warning existant et n'autorise aucun effet. Coût O(log p+log n).

Les deux anciens appels aux labels sont adaptés à acteur/action/path sans changement de valeurs. Les deux mappings d'intention des Send sont calculés avant prepare_dispatch et réutilisés dans leur check : SteerCurrent→Steer, InterruptAndStart→Interrupt, autre→None, valeurs identiques.

## Frontières des traces

Toutes les catégories fermées de prepare_dispatch sont tracées pour l'intention ciblée neuve, y compris duplicate_content/quarantined : le rejeu idempotent est déjà rendu en amont. Aucun détail sensible de projet, SQL ou corps n'entre dans outcome.

Les faits présents après la garde utilisent directement record_guidance_outcome : destinataire indisponible, sérialisation, admission d'exécution illisible/terminale/non persistée, begin_send_delivery_with_guidance échoué, record_message/track_reply_cycle échoué, génération/révision/état-cible/connexion du Control, mark_control_dispatched échoué, persistance du premier admitted échouée, cancel_request refusé/non persisté. Les erreurs génériques de persistance sont nommées refused:persistence_failed ; les autres utilisent leurs catégories fermées.

Canon, réservation, résultats des gardes, quotas, reçus et reprises, mutation de cancel, pause interne et envoi physique sont conservés. Les refus authorize déjà tracés ne reçoivent pas une seconde trace. Les JSON invalides, invalid_intent, envelope_mismatch et reçus rejoués restent exclus. Après admitted réussi, aucun refus nouveau n'est ajouté : l'échec fournisseur garde admitted et l'issue indéterminée existante.

Le suffixe des tests existants est byte-identique à la source ; aucun oracle nouveau ou modifié. ROOT coordonne l'extension de guidance156_rate_refus_non_compte avec son owner tests.
