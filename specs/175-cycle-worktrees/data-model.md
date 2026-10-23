# Données — Session 175
Aucune migration. WorkspaceRecord et les échanges worker ne changent pas.
## Projection de conservation
workspace.retention : objet dérivé {code, message, recovery}. Codes fermés : task_active, stop_unconfirmed, report_unavailable, changes_present, branch_changed, commits_to_review, review_required.
Aucun booléen removable. Même contrôle du parent autorisé que le statut existant.
Le rappel du résultat invite à relire le statut ; il ne fige pas le motif de conservation.
## Inventaire local
Objet version=1 : repo, base_ref, capacity, processus, note, worktrees.
capacity : scope (cwd/.worktrees), scoped_count, git_total (principal inclus), limit=128, warning_at=96, state normal/warning/limit et limite (texte sur les réservations non observées).
processus : complet ou inconnu. Un échec du registre donne une erreur globale version=1, erreur=inventaire_git_indisponible, code de sortie1, sans compte fictif.
Entrée worktrees : path, head, locked, prunable, motifs (liste de codes), recovery (liste de consignes).
Motifs : principal, hors_perimetre, locked, nested, dirty_ignored, non_integre, recent, processus_actif, lecture_inconnue, candidat_a_examiner, lien_symbolique, chemin_absent.
candidat_a_examiner signifie observations Git/processus locales satisfaites ; tâches/fils restent à vérifier. Aucun rapport n'autorise un retrait futur sans relecture.
