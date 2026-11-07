# Contre-revue du plan 241

Date : 7 novembre 2026. Demande native -02 ; tâche
cf827b77-4d3a-4043-bade-2184118bdc85. Moteur Claude, modèle choisi par Bridget
claude-haiku-5-5, machine SYS. Parent Codex. Lecture statique de la base04ad et
plan intégral inclus au brief ; pas d'accès prétendu aux originaux Mac.

Demande -01 : Mac refusé pour saturation CPU/charge, not_recorded/tasknull.
Demande -02 : résultat disponible, cleanup_done=true. Deux lectures MCP du
résultat ont échoué sur JSONL/raccordement. Résultat lu dans la base native en
mode lecture seule pour ne pas bloquer le travail indépendant. ACK natif acquis après retour du lien ; reçu conservé, sans prétendre une lecture humaine.

Verdict reçu : BLOCKED pour A.4/A.5, APPROVE_WITH_CHANGES pour le reste.
La revue ne décide pas du produit ; chaque objection est vérifiée.

| Objection | Vérification | Retenue | Résolution |
|---|---|---|---|
| Révocation sans source hors HTTP | delegation.rs:2760-2800 et native_delegation.rs:4566,6970-6985 | Oui sur l'exigence, non sur l'absence de source | Révocations natives persistées déjà présentes. Contrat explicite dans contracts/admission.md ; tests de révocation avant Release obligatoires. Aucun retrait silencieux du contrôle. |
| Fin du tour ferait tomber la route | origin_proof.rs:283 et registre de présence | Non établi | La route appartient au fil, pas au tour ; la preuve HTTP exige le tour actif. Chronologie conservée, absence de route non inventée. |
| Entretien avant inscription auxiliaire | daemon.rs:19881 et pending_post_response_controls:7638 | Oui | Réutiliser le traitement après réponse si aucune dépendance ne l'interdit ; mesurer le chemin réel. |
| Branche locale liée à l'hôte, pas au chemin | mapping_authority et for_origin, policy_directories | Oui | Comparer origine/cible provisionnées, préserver uniquement les répertoires déjà attestés avec contrôles de chemins. |
| Plafonds supplémentaires | bornes origin_proof, workspace_git, protocol, t3code, handoff, service_limits | Oui | Les inclure dans l'inventaire demandé ; une borne ne devient pas justifiée par sa seule présence. |
| Contrat d'outillage absent | CandidateProbeV2 ne contient que workload | Oui | Compléter le protocole négocié existant et ses tests ; commandes de sonde en lecture seule, sans installation. |
| Marqueur d'autorité reporté | Task:1044 et snapshot transport:23 | Oui | Champ interne optionnel sur Task ; contrat écrit avant les tâches, compatibilité réelle exigée avant activation. |

Décision responsable : les deux conditions bloquantes deviennent des exigences
de réalisation et de test précises avec sources natives identifiées. Elles ne
sont pas déclarées implémentées. Aucun besoin de supprimer les révocations,+d'ajouter une table ou de redémarrer T3 pour établir ce contrat de mission.

Minimalisme : réutiliser les contrôles déjà appliqués aux missions native_wrapper
et les magasins existants, sans mécanisme de bail parallèle. Potentiel de
simplification : réattestation redondante après admission, pas encore chiffré.
Responsabilité future : distinction de durée documentée, marqueur explicite,
ancien travail inchangé ; aucun changement de permissions caché dans un défaut.
