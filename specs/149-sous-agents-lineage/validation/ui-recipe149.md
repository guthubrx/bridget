# Recette UI web 149 - Lineage (T040, document canonique)

Date : 2026-10-10. Rédacteur : owner documentaire (Haiku 5.5). Ce document fait la synthèse des rondes existantes. Il ne relance aucune recette et ne refait aucune capture. Aucune case n'est cochée.

Sources lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149.md` (r2), `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r3.md` (r3), `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md` (r4, la plus récente).

## 1. Verdict et portée

- Verdict de la ronde r4 : APPROVE, zéro finding actif. Ce verdict vaut pour la recette UI web avec daemon fixture.
- T040 reste non coché. Le cochage revient au principal, après revue.
- Revue T041 (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md`) : le critère de T040 est rempli à son niveau cible. Ce critère est une recette UI web sur serveur isolé avec daemon fixture, journal, nested, statuts, modèle, arrêt, reconnexion et preuves visuelles. SC001 et T037 sont des preuves séparées. Elles ne rendent pas T040 incomplète.
- Preuve valide : l'interface web Lineage, le journal, l'arrêt et la reconnexion, avec un serveur T3 réel et un daemon simulé.
- Preuve non valide : la coque desktop, le mobile, iOS, le moteur natif de bout en bout, SC001, T037 et le MCP privé authentifié.

## 2. Ce qui est réel et ce qui est simulé

| Couche | État |
|---|---|
| Serveur T3 du worktree (`apps/server`, Node 24) | RÉEL |
| Auth par jeton scopé et ticket WebSocket | RÉEL |
| Projection SQL privée, `BridgetLineage`, garde lecture seule des fils virtuels | RÉEL |
| UI web du worktree (Vite+) ouverte dans la preview T3 | RÉEL |
| Daemon Bridget : `bridget_fixture.mjs` sur un magasin JSON privé | SIMULÉ |
| Tâches, journaux, résultats : jeu `[recette149]` | SYNTHÉTIQUE |
| Modèle `glm-5.3-flash` affiché | Donnée de fixture. Aucun modèle n'a tourné. |

Aucun modèle n'a tourné. Aucune app T3 Desktop n'a été lancée. Aucun service de production n'a été touché. Les tests de clic utilisent la preview T3, sans Playwright ni Chrome.

## 3. Compteurs de la dernière ronde (r4)

Règle : on garde le dernier compte par suite. Les rondes ne s'additionnent pas, car elles rejouent souvent les mêmes batteries.

| Suite | Résultat r4 | Source |
|---|---|---|
| Tests serveur 149 (5 fichiers) | 120 PASS, 0 FAIL | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md` |
| Tests UI et client ciblés (journal, ThreadRelationshipsControl, client-runtime) | 22 PASS, 0 FAIL (3 fichiers, sans le mobile). Rejoués par la revue T041 avec le journal mobile : **24 PASS, 0 FAIL, 4 fichiers**. | idem et `review-final149.md` |
| `BridgetLineage149.test.ts` seul | 23 PASS, 0 FAIL | idem |
| Matrice réseau `matrix149.mjs all` | 38 PASS, 0 FAIL | idem |
| Rejeux `replay_r3.mjs all` | 23 PASS | idem |
| Rejeux `replay_r4.mjs all` (nouveau) | 22 PASS | idem |
| Mesures navigateur F2 | 9 PASS, 9 captures | idem |
| `tsc --noEmit` serveur | 16 erreurs, toutes de base, 0 dans les fichiers 149 | idem |
| `tsc --noEmit` web | 10 erreurs, toutes de base, 0 dans les fichiers Bridget | idem |

Le compte de 22 tests UI ciblés ne représente pas la totalité des tests web du dépôt. `tsc --noEmit` a été rejoué par la revue : contracts, client-runtime, provider-core et mobile à 0 erreur ; serveur 16 et web 10, tous dans des fichiers non touchés par le diff.

Compteurs SQL en fin de recette r4 : `runs`, `run_attempts`, `provider_turns`, `provider_sessions`, `provider_threads`, `bindings`, `runtime_requests` et `workflows` valent 0. Il reste 4 lignes `effect_outbox`. Ce sont 2 fois `terminal.cleanup` et `preview.cleanup`, créées par les deux `thread.delete` de la ronde. Elles ont statut `succeeded`. Aucun effet fournisseur n'a eu lieu.

## 4. Résultats par exigence de S149-28

Les lignes marquées r2 ou r3 viennent des rondes antérieures. Elles restent valides pour leur périmètre. Les lignes marquées r4 sont la preuve la plus récente.

| Exigence | Résultat | Preuve |
|---|---|---|
| Sous-agent visible dans Lineage (panneau « Lineage · 1 running ») | PASS (r2) | captures r2 01, 02 |
| Sous-agent absent de la barre latérale, même avec 134 tâches projetées | PASS (r2 et r3) | captures r2 01 et 11 ; capture r3 01 (3 fils visibles sur 134) |
| Journal sans composeur (0 `textarea`, 0 `contenteditable`), barre « Tâche Bridget · lecture seule » | PASS (r2 et r3) | capture r2 02 ; ronde r3 |
| Journal live : événements ajoutés sans rechargement | PASS (r2) | capture r2 02 |
| Modèle, effort et statuts natifs affichés (libellé « Attend ses enfants », etc.) | PASS (affichage seulement) | captures r2 02, 05, 07 |
| Nested et statut mis à jour en direct, parent en « Stopped » | PASS (r2) | capture r2 05 |
| Arrêt natif racine et imbriqué : `lineage cancel`, `request_id` UUID distincts, reçu `cancelling` | PASS (r2 et r3) | captures r2 03 ; `store.json` ; ronde r3 (captures r3 03) |
| Aucun ProviderTurn, Run, session ni tour produit par la vue | PASS (compteurs à 0) | ronde r2 et r4 |
| Résultat terminal lu, sans bouton « Arrêter » | PASS (r2) | capture r2 07 |
| Reconnexion après coupure du flux, sans doublon | PASS (r2) | captures r2 03, 04 |
| Lacune de journal : « Journal incomplet : séquences 3 à 5 » | PASS (r2) | capture r2 10 |
| Journal indisponible : « Dernier état connu » ; le contenu lu reste visible | PASS (r2 et r3) | capture r2 08 ; capture r3 07 |
| Sélection d'un enfant puis « Ouvrir le parent » | PASS (r2) | captures r2 01, 02, 05 |

### Résultats propres à la ronde r4

| Point vérifié | Résultat | Capture |
|---|---|---|
| Contexte forgé (jeton read, projet étranger, vrai fil) : refus `project_mismatch`, aucune mutation, Lineage reste disponible, « Arrêter » actif | PASS | `06` puis `07` |
| Vraie autorité dégradée (projet réel différent) : Lineage « indisponible », « Arrêter » absent, historique conservé | PASS | `08` |
| Reprise après « Reconnecter Bridget » : Lineage disponible de nouveau | PASS | `09` |
| Dernière ligne du journal au-dessus de la barre basse, en défilement maximal | PASS | `01` (1280), `02` (480), `03` (375) |
| Clic réel sur « Arrêter » de la tâche imbriquée (titre long, carte « Thread details » ouverte, 1280 px) : le clic passe, la tâche passe en `cancelling` | PASS | `04` |
| Résultat terminal et journal en défilement maximal | PASS | `05` |

Le constat r2 « Arrêter masqué à 1280 px » est clos. La ronde r3 a mesuré le bouton atteignable (`elementFromPoint`). La ronde r4 le confirme.

Les points 1024 px et 768 px ont été testés en r3, et pas en r4. Les tailles 1024 et 768 ne sont donc pas rejouées dans la dernière ronde. iOS et desktop ne sont pas testés.

### Hauteur de la barre basse mesurée (r4, défilement maximal)

| Largeur x hauteur | Hauteur de la barre | `paddingBottom` du conteneur | Écart dernière ligne / barre | Composeur |
|---|---|---|---|---|
| 1280 x 800, carte ouverte | 60 px | 72 px | 12 px | aucun |
| 480 x 800 | 54 px | 66 px | 12 px | aucun |
| 375 x 800 | 54 px | 66 px | 12 px | aucun |
| Redimensionnement 375 vers 1280 sans rechargement | 60 px | 72 px | 12 px | aucun |
| Après « Reconnecter » (démontage et remontage du flux) | 60 px | 72 px, pas de doublement | 12 px | aucun |
| Tâche terminée avec résultat lu | 60 px | 72 px | 12 px | aucun |

Le `paddingBottom` vaut `calc(0.75rem + Npx)`, où N suit la hauteur réelle de la barre. Il se mesure à chaque redimensionnement. Le r3 perdait environ 40 px sous la barre. Le r4 n'en perd plus.

## 5. Captures à consulter

Les captures sont des fichiers privés. Les chemins sont absolus et copiables. Les liens Markdown pointent vers les fichiers réels.

### Ronde r4 (9 captures)

Dossier : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/`

1. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/01-f2-racine-scroll-max-1280-derniere-ligne-au-dessus-barre.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/01-f2-racine-scroll-max-1280-derniere-ligne-au-dessus-barre.png))
2. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/02-f2-racine-scroll-max-480-adapte-sans-composeur.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/02-f2-racine-scroll-max-480-adapte-sans-composeur.png))
3. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/03-f2-racine-scroll-max-375-adapte-sans-composeur.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/03-f2-racine-scroll-max-375-adapte-sans-composeur.png))
4. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/04-f2-imbrique-titre-long-1280-carte-ouverte-arreter-cliquable.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/04-f2-imbrique-titre-long-1280-carte-ouverte-arreter-cliquable.png))
5. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/05-f2-tache-terminale-resultat-journal-scroll-max-1280.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/05-f2-tache-terminale-resultat-journal-scroll-max-1280.png))
6. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/06-f1-hote-avant-appel-forge-lineage-disponible.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/06-f1-hote-avant-appel-forge-lineage-disponible.png))
7. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/07-f1-hote-apres-appel-forge-lineage-toujours-disponible.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/07-f1-hote-apres-appel-forge-lineage-toujours-disponible.png))
8. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/08-f1-autorite-reelle-project-mismatch-natif-lineage-indisponible.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/08-f1-autorite-reelle-project-mismatch-natif-lineage-indisponible.png))
9. `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/09-f1-reprise-apres-reconnecter-lineage-disponible.png` ([voir](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/09-f1-reprise-apres-reconnecter-lineage-disponible.png))

### Ronde r2 (11 captures, historiques pour le parcours complet)

Dossier : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots/`

Les fichiers sont `01-lineage-parent-enfant-sidebar.png`, `02-journal-enfant-actif-live-sans-composer.png`, `03-journal-stop-annule-flux-coupe-contenu-conserve.png`, `04-reprise-reconnecter-sans-doublon.png`, `05-nested-statut-live-parent-stopped.png`, `06-constat-bouton-arreter-masque-1280px.png` (constat clos, voir section 4), `07-terminal-resultat-lu.png`, `08-offline-journal-indisponible-tache-echouee.png`, `09-reprise-apres-pannes-fil-hote.png`, `10-gap-journal-incomplet-seq-3-5.png`, `11-constat-fils-orphelins-133-previous-agents-sidebar-masquee.png`.

### Ronde r3 (10 captures, historiques)

Dossier : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r3/`

La capture `10-FINDING-contexte-forge-read-rend-lineage-indisponible.png` documente le finding F1 de r3. Ce finding est corrigé en r4 (captures r4 06 et 07).

## 6. Constats et corrections

| Constat | Statut | Par qui |
|---|---|---|
| F1 (r3, sûreté) : un contexte forgé rendait Lineage indisponible pour toute la racine | Corrigé. Vérifié en r4 (5 nouveaux tests, mutation testée, captures 06 et 07). | Correction de Sol dans `BridgetLineage.ts` |
| F2 (r3, interface) : la barre basse masquait la dernière ligne du journal | Corrigé. Vérifié en r4 (mesures du tableau ci-dessus). | Correction de Sol dans `BridgetTaskJournal.tsx` |
| Constat r2 : « Arrêter » masqué à 1280 px, carte ouverte | Clos en r3, confirmé en r4 | Vérification sur l'interface |
| `snapshot_changed` épuisé dégrade les 135 fils d'un coup | Observation. Aucune correction demandée. | À trancher par l'owner |
| Après une panne globale, la bannière watch exige un clic « Reconnecter Bridget » | Observation. Comportement de design existant. | À trancher par l'owner |
| Le message « Le contenu déjà lu est conservé » s'affiche même sans lecture (cas T4) | Observation. Non corrigé. | À trancher par l'owner |
| Un compte de fils a varié (137 puis 138) lors d'une lecture SQL | Non reproduit. Aucun événement de suppression. | Non qualifié |
| Fixture : le CLI ignore `--t3-thread`. S149-19 n'est pas prouvé. | Limite de fixture | Hors périmètre de cette recette |
| Fixture : un second fil hôte du même projet ne peut pas être lié (ZD-1) | Limite de fixture. Le cas est une observation. | Hors périmètre de cette recette |

Un constat r2 sur la fixture est clos. Le CLI de fixture ne lançait pas le watch correctement, et la sortie `--follow` répétait des événements. Ce point est corrigé dans la fixture.

## 7. Limites à lire avant toute conclusion

1. Le daemon Bridget est simulé (`bridget_fixture.mjs`). Cette preuve ne valide pas le moteur natif.
2. Les données sont synthétiques (`[recette149]`). Le modèle affiché est une donnée de fixture. Aucun modèle n'a tourné.
3. SC001 n'est pas validé. Une délégation réelle de bout en bout n'a pas eu lieu.
4. Le MCP privé authentifié n'est pas prouvé. La matrice valide les scopes côté RPC, pas le MCP privé.
5. T036, T037, T038 et T039 ne sont pas prouvés par cette recette. Leurs preuves natives relèvent d'autres owners, encore en cours.
6. Le mobile n'est testé qu'en jsdom. Le viewport de 375 px est une fenêtre de navigateur, pas une vraie coque mobile.
7. iOS et la coque desktop ne sont pas testés.
8. La recette ne remplace aucune preuve de service. Elle vaut pour l'interface et le serveur T3 uniquement.

## 8. Commandes de reproduction (référence, non relancées)

Ces commandes décrivent la ronde r4. Elles ne sont pas à relancer pour cette synthèse. Les jetons ne sont pas notés ici.

```
export C=<cache privé de la recette, par exemple /Users/moi/.cache/bridget149-ui.r4>
export RECIPE149_CACHE=$C RECIPE149_T3_PORT=<port éphémère> RECIPE149_WEB_PORT=<port éphémère>
# tests (racine du worktree)
./node_modules/.bin/vp test run apps/server/src/bridget/BridgetLineage149.test.ts apps/server/src/bridget/BridgetReader149.test.ts apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts apps/server/src/orchestration-v2/ProjectionStore.bridget149.test.ts apps/server/src/orchestration-v2/ThreadLaunchService.bridget149.test.ts
./node_modules/.bin/vp test run apps/web/src/components/BridgetTaskJournal149.test.tsx apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx packages/client-runtime/src/state/orchestration.bridget149.test.ts
# recette (depuis ui-recipes/)
node fixture_store_init.mjs $C/store.json $C/proj ; zsh run_recipe_server.sh start
node --no-warnings matrix149.mjs all ; node --no-warnings replay_r3.mjs all ; node --no-warnings replay_r4.mjs all
```

Le dossier de travail de la recette est `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipes/`.

## 9. Nettoyage (état déclaré en r4)

Les processus de recette ont été arrêtés un par un, avec contrôle d'identité. Les ports de recette sont libres. Les jetons et les codes d'appairage ont été supprimés. La base privée et le magasin JSON sont conservés sous `/Users/moi/.cache/bridget149-ui.r4`. Aucun job différé n'est programmé.

## 10. Ce qui reste à faire pour T040

- Décider du cochage T040 après revue du principal. Ce document ne coche rien.
- Décider du sort des observations de la section 6.
- Ne pas étendre cette preuve à la coque desktop, à SC001 ou au moteur natif. Ces points relèvent de T036 à T039.
