# Bilan de prélivraison - session 149

Date : 2026-10-10. Auteur : agent documentaire Claude Haiku 5.5 (medium). Périmètre : documentation seule. Aucun test, build, Cargo, Git, service ni sous-agent n'a été lancé. Les chiffres viennent des rapports cités.

## Statut

- **T001 à T042 : validés au principal.** Le principal coche les 42 cases de `tasks.md`.
- **T043 à T045 : en attente.** Commits et fusion, paquets et installation, reçu final et nettoyage.
- **Non livré.** Aucun 149 n'est installé. La production 148 reste en place.

## Preuves actuelles

- **Binaire candidat :** release `abc850858975`, sha256 `abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8`.
- **Production revue :** 111 fichiers, empreinte `b2b87458cf3cec7989debeb91352b838c417c3eb48329d57099bec4c9a0d5f29`. Revue des sources : SOURCE_ONLY_APPROVE (`validation/native-restart-final-source-review-sonnet-r2.md`).
- **Recettes réelles sans nouveau grant :** GLM `glm-5.3-flash` et Codex `gpt-6.1-sol` (effort high) (`validation/native-real-recipes-sonnet-r5.md`). Smoke r6 : `validation/native-real-smoke-sonnet-r6.md`. Redémarrage avec parent Codex vivant : `validation/native-real-restart-sonnet-r7.md`, 29 oracles OK, 0 échec.
- **Réseau et interop :** `validation/native-network-proofs-r3.md` (T036 fermé). `validation/native-network-proofs-r4.md` : O4.6, E2 et F3 fermés, 16 contrôles OK, 0 FAIL.
- **Interface :** `validation/ui-native-recipe149-sonnet-r1.md`, avec daemon réel et fournisseurs fictifs explicites.
- **Frontière T3 :** `validation/t3-final-boundary-tests-sonnet-r2.md` (G3 et G5 fermés).

## Baselines

| Contrôle | Résultat |
|---|---|
| Rust natif (r9) | 1840 PASS = 1507 daemon + 333 transport, 0 FAIL final. 19 tests nouveaux (17 + 2). |
| Rust, charge | 3 échecs initiaux sous charge, puis 3 PASS isolés. |
| SC005 (bench ancien) | Non concluant. Ce bench ne mesure pas l'opt-out SPEC149. |
| Recompte r8 | 1822, et non 1825. |
| `tsc` T3 | Serveur 16 erreurs, web 10 erreurs, 0 nouvelle. |
| Lint T3 | 0 erreur, 908 avertissements dont 7 nouveaux non bloquants. |
| Format T3 | 48 PASS. |
| Interface | 25 PASS sur 4 fichiers (décision du principal). La revue T041 comptait 24 sur les mêmes fichiers. Écart non tranché. |
| Frontière T3 (r2) | 262 tests, 16 fichiers. |

Ces chiffres ne prouvent pas un GREEN global exhaustif. La suite T3 complète n'est pas promise.

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
