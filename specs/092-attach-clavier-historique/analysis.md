# Analyze 092 — 2026-09-06

Exécution manuelle du protocole speckit-analyze : scripts de prérequis absents,
sélection explicite via .specify/feature.json. Spec, plan, tâches et constitution
chargés ensemble après génération des tâches. Phase d'analyse sans modification
de code. Correction d'artefact séparée autorisée par le pipeline utilisateur.

| ID | Sévérité | Constat initial | Résolution avant délégation |
|---|---|---|---|
| A1 | medium | Échap enrichi peut casser le repli Échap puis Entrée annoncé | Ajouter CSI 27u au contrat ; T002 couvre les contrôles enrichis |
| A2 | medium | Le tampon est créé par connexion, pas par invocation | Remonter son ownership à run_with_input ; T005 teste conservation et nouvelle invocation vide |
| A3 | medium | Écho différentiel si stdout redirigé, incompatible avec rappel visuel | Navigation réservée stdin ET stdout TTY ; sinon flèches consommées sans effet, T005 |

Seconde passe : aucune ambiguïté bloquante, aucun NEEDS CLARIFICATION, aucun
doublon d'autorité ou protocole. La couverture des 8 FR et des 6 SC figure au bas
de tasks.md, 14/14 couverts par 9 tâches, 0 tâche orpheline, 0 critical/high.

Constitution XIX/XX : pas de dépendance, migration ou abstraction publique ; la
duplication est évitée par extension d'InputBuffer/RawTerminal. Les états privés
portent des invariants clavier et mémoire. Charge future limitée et vérifiable.
Le worktree dédié est réutilisé sous nouvelle branche pour conserver les droits
effectifs, sans toucher au checkout principal ni capturer le fichier de preuve.

Limites explicites : pas de garantie matérielle Shift+Entrée si mêmes octets que
Entrée ; une injection PTY ne prouve pas une frappe physique. Revue cross-provider
indisponible dans l'annuaire. Aucun commit/déploiement automatique.

Verdict : PASS — l'équipier peut implémenter T001–T008 ; T009 reste au pilote.
