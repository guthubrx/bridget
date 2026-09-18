# Plan de tests 103

Statut : 23 scénarios PLANIFIÉS, zéro exécuté. Les assertions ci-dessous
sont des gates de l'implémentation, pas des succès annoncés.

## Harnais et méthode

Fixtures synthétiques, vraie baseSQLite et sockets temporaires aux frontières ; aucun
fournisseur réel ni API payante. L'identité doit suivre099 : RegisterAuxiliary/proof avant
ClientHello ou requête enregistrée ; pas d'UUID pris depuis un fichier sans preuve.
Exprimer chaque scénario sous forme Given/When/Then dans les tests Rust natifs :
Given fixture initiale, When appel réel, Then état/résultat observable.
Les tests de performance doivent rapporter matériel, buildmode, taillecorpus, p95 et mémoire.
Un test qui ne mesure que ses mocks n'est pas une preuve de transport ou d'accès.

Avant un test processus, inspecter ses helpers de nettoyage : SIGKILL et kill de groupe
non vérifié sont interdits. Home/socket/base/journaux isolés ; ne jamais lancer le daemon
sur /Users/moi/Nextcloud/10.Scripts/64.bridget ni sur le home de production /Users/moi/.cache/bridget-core.
Arrêt ciblé d'un PID propre vérifié (jamais Firefox), attente et collecte du code de sortie.

## Matrice

| ID | Story | Given / When | Then observable | Couverture |
|---|---|---|---|---|
| S01 | US1 | Dossier minimal | Objectif et résumé seuls donnent un corps v1 stable ; sections absentes par défaut. | FR-001, FR-002, SC-001 |
| S02 | US1 | Prévisualisation | Daemon inexistant : aperçu valide ; zéro socket/envoi et aucun ID inventé. | FR-003, SC-002 |
| S03 | US1 | Structure stricte | Champ inconnu, null, version imposée, faux type ou texte blanc : invalid_params avant transport. | FR-001, FR-007, SC-004 |
| S04 | US1 | Taille Unicode | Corps final exactement16384octets accepté,16385refusé ; emoji/échappement inclus. | FR-007, SC-004 |
| S05 | US1 | Bornes listes | Chaque maximum de liste/champ testé àN puisN+1, pas de troncature. | FR-007 |
| S06 | US1 | Envoi réel synthétique | Un destinataire attesté reçoit le corps exact ; pas d'appel de modèle. | FR-004, FR-010 |
| S07 | US2 | Sources inertes | Références file/url/journal/thread/artifact/messages : aucun accès disque/HTTP/lecture de source. | FR-005, FR-006, SC-006 |
| S08 | US2 | Référence invalide | Chemin relatif, URL avec credentials, séquence inverse, UUID invalide refusés. | FR-005, FR-007 |
| S09 | US2 | Conservation | Retrouver body identique après réouverture DB ; purge configurée l'enlève normalement. | FR-006, FR-011 |
| S10 | US2 | Déclarations | Évidence/hypothèse/limites transmises sans requalification en résultat certifié. | FR-002, FR-014 |
| S11 | US2 | Contenu hostile | @all, commandes shell, HTML et faux rôles : aucun effet ; rendu terminal inerte. | FR-014, SC-006 |
| S12 | US3 | Rejeu | Même clé/date/draft envoyé dix fois : une opération et un message ledger. | FR-008, SC-002 |
| S13 | US3 | Contrat différent | Modifier summary sous même clé : envelope_mismatch, première opération intacte. | FR-008 |
| S14 | US3 | Perte du reçu | Couper après réservation : reçu inconnu conserve clé/date ; rejeu identique sans doublon. | FR-008 |
| S15 | US3 | Réponse facultative | Défaut reply=false sans demande suivie ; true crée seulement la demande099 normale. | FR-009 |
| S16 | US3 | Pas de contournement | DND/offline/occupé/instance révoquée : statuts099 inchangés, pas de relance ni nouveauUUID. | FR-009, FR-010, FR-013 |
| S17 | US3 | Mise à jour | Nouveau dossier explicite avec nouvelle clé ; ancien corps non muté. | FR-008, FR-013 |
| S18 | US4 | Parité | Même objet CLI/MCP→mêmes octets ; preview/send et erreurs testés. | FR-012, SC-003 |
| S19 | US4 | Stdin borné | 64Kio accepté syntaxiquement,64Kio+1 refusé avant parsing ; EOF/JSON invalide diagnostic. | FR-007, FR-012 |
| S20 | US4 | Catalogue et anciens clients | Ajout seul outil nommé, aucun serveur global autorisé ; ancien wrapper lit le texte. | FR-012, FR-013 |
| S21 | US4 | Recette de documentation | Trois dossiers synthétiques : lecteur trouve objectif/état/prochain pas/limites sans historique. | FR-001, FR-002, FR-011, FR-012, SC-001 |
| S22 | US4 | Performance | 200 rendus bornés p95<100ms et mémoire O(B) ; mesurer sans réseau/modèle. | SC-005 |
| S23 | US4 | Compatibilité du rendu v1 | Fichiers dorés Unicode, slash, contrôles, listes vides et ordre : octets inchangés entre versions ; source_label non routable. | FR-008, FR-012, SC-003 |

## Commandes futures

Depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation, APRÈS implémentation et audit du harnais :

```sh
cargo fmt --all -- --check
cargo test -p bridget-daemon --lib spec103
cargo test -p bridget-daemon --test handoff_103_test
cargo clippy --workspace --all-targets -- -D warnings
```


Faire correspondre les noms de tests au filtre spec103, et vérifier qu'un filtre ne lance
pas zéro test. Régressions089/094/099/100 et autres surfaces touchées seulement après audit
de leur isolation ; ne pas lancer aveuglément tout workspace en release sur ce poste partagé.
Les tests fonctionnels ne nécessitent pas T3 vivant. Toute recette réelle ou installation
future requiert une nouvelle autorisation, pas un héritage de l'accord donné pour101.

## Négatifs indispensables

Erreurs avant effets, identité manquante/révoquée, source inconnue ou interdite, caractères
Unicode aux limites, interruption et issue inconnue, ancien client et ancien daemon.
Conserver erreurs/status sans reformulation en réussite métier. Aucun secret/corps dans logs.
