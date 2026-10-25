# Bilan de prélivraison - session 149

Date : 2026-10-10. Auteur : agent documentaire Claude Haiku 5.5 (medium). Périmètre : documentation seule. Aucun test, build, Cargo, Git, service ni sous-agent n'a été lancé. Les chiffres viennent des rapports cités.

## Statut

- **T001 à T042 : validés au principal.** Le principal coche les 42 cases de `tasks.md`.
- **T043 à T045 : en attente.** Commits et fusion, paquets et installation, reçu final et nettoyage.
- **Non livré.** Aucun 149 n'est installé. La production 148 reste en place.

## Preuves actuelles

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/prelivraison149.md#extrait-1).

## Baselines

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/prelivraison149.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/prelivraison149.md#extrait-3).

## Limites acceptées

- **Processus :** les workers 15590 et 15618 ont été nettoyés par SIGUSR1, un par un, après vérification d'identité. Aucun SIGKILL.
- **G6 et G9 :** l'analyse parse v1 et le rollover de seq sont couverts par revue de source et tests de forme. Pas de test direct.
- **S149-21 (opt-out) :** les tests 147 et 148 déjà en régression couvrent l'opt-out et le MCP désactivé. Aucun nom de test n'est inventé.
- **Scénarios `readOnly` et annulation GLM :** non joués en réel. Limites optionnelles. Pas de matrice réelle exhaustive.
- **Fournisseur ignorant SIGTERM plus de 8 s :** le démarrage est refusé et le marqueur reste. Comportement conforme au guide d'exploitation. Pas de forçage humain.
- **E4 (SIGKILL externe du fournisseur) :** la fin de tour reste inconnue jusqu'au tour suivant. Pas de garantie universelle.
- **Liens `agent_links` après reprise :** observés en r4, sans impact testé. Note non bloquante.
- **Observateur automatique (r7) :** le script a échoué. La preuve retenue est le cycle de vie r6 (PID, `ps` et TUI).
- **Drapeau CLI `yolo` (ancien registre) :** non retesté dans cette ronde. La limite OS négative réelle de p5 est conservée.
- **Scénarios :** 33 scénarios de niveaux U, P, R et M, plus la source. Ils ne sont pas tous réels.

## Décision et livraisons non faites

- **Décision du principal :** GO sur T001 à T042 pour les niveaux vus. Aucun élargissement.
- **Non fait :**
  1. Commits et fusion (T043).
  2. Paquets finaux, depuis les commits fusionnés, puis installation (T044). Les fichiers 148 sont sauvegardés dans `/Users/moi/.cache/bridget-install149.8atzwz`.
  3. Reçu final et nettoyage (T045).
- **Production :** les processus 58394, 58396 et 58468 (démarrés le 9 octobre) restent inchangés. Aucun redémarrage ni activation différée.
