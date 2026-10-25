# Archives techniques de l’adaptateur T3

Document d’origine : [specs/143-echanges-discrets/implementation.md](../../../../../../specs/143-echanges-discrets/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Étape | Preuve | État |
|---|---|---|
| Préflight | Sync projet appliqué ; scripts/templates locaux vérifiés absents ; protocole documentaire de remplacement | Réalisé |
| Sources | Frontend isolé base `5724eb7f12e4556327efa14f51f43b9caf404e25`, quatre fichiers existants suffisants | Périmètre fixé |
| Exploration | Formes Codex/Claude `bridget_send` attestées, refus/inconnu → natif | Consignée |
| Gate | Réutilisation PASS, huit items, cinq cases gate, lecture intégrale principale | Validé avant tâches |
| Analyze | Deux passes, aucun finding bloquant ou restant | GO implémentation principal |
| Baseline | Équipier : deux suites existantes, 294/294 PASS en 4,92 s | Baseline seulement, pas preuve finale143 |
| T001 RED | Équipier, preuve transmise par le principal : UI 7 échecs/4 PASS ; logique 4 échecs/24 PASS | Échecs comportementaux consignés après baseline294 |
| T002 Projection | Formes Codex/Claude, destinataires et replis couverts dans 252 tests logique GREEN finaux | Vérifiée |
| T003 Entrées | Styles sans cadre, contenu et régressions dans les tests UI GREEN | Vérifiée |
| T004 Sorties | Rendu compact/natif et garde-fous dans les tests UI GREEN | Vérifiée |
| Durcissement de revue | Deux réserves corrigées après 9 cas logique et 3 UI RED, puis GREEN ; fonction publique 42 lignes, validation privée 35, primitive record et contrôle flags réutilisés | Preuves transmises par l'équipier |
| T005 Interactions | Deux suites GREEN finales : 346/346, dont 252 logique et 94 UI | Vérifiée |
| T006 Contrôles | Format quatre fichiers PASS ; lint zéro erreur/22 warnings de baseline ; types web PASS ; build final PASS en 32,44 s avec avertissement de chunk existant ; diff PASS | Vérifiés sur code gelé final |
| T007 Recette | Principal : navigateur sur clone isolé, transparent/border0, clavier/focus/copie, 320px, Codex/Claude, refus/inconnu et ordinary | PASS, cochée après preuves |
| Contre-revue finale | Interne APPROVE après RED/GREEN ; huit cas manuels réellement exécutés en mémoire, lecture seule, par le relecteur | Compteur distinct des346 tests Vitest ; autre fournisseur indisponible |
| Converge1 | Principal, lecture seule dix FR/cinq SC sans écart ; tâches byte-identiques SHA043ef959… | CONVERGED passe1 |
| T008 Audit | 509 s, grade A/100 sur le diff de quatre fichiers ; deux MEDIUM historiques corrigés, aucun résidu ; 346 tests rejoués en 2,97 s | PASS, cochée après preuves |
| Validation audit | Validateur de session rejoué par l'auditeur et le principal : exit0, zéro erreur/zéro warning | PASS |
| Converge2 | Principal,09:22:07 CEST, dix FR/cinq SC sans écart ; tâches byte-identiques SHA939891be… et quatre sources inchangées | CONVERGED passe2 |
| Arrêt aperçu | Onglet isolé fermé ; backend93227/session81143 et frontend61090/session15374 arrêtés via Ctrl-C après identification ; ports13916/5876 sans listener | Clone arrêté proprement, conservé |
| Protection T3 | PID85017 toujours démarré à07:21:41, aucun restart de l'application active | Préservé |

<a id="extrait-2"></a>

## Extrait 2

Commande baseline depuis `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/` : `./node_modules/.bin/vp test run --project unit src/components/chat/MessagesTimeline.logic.test.ts src/components/chat/MessagesTimeline.test.tsx`.

<a id="extrait-3"></a>

## Extrait 3

Recette : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/ui-recipe.json`. Aperçu natif explicitement indisponible ; remplacement par navigateur sur `http://127.0.0.1:5876`, clone `/Users/moi/.cache/t3-spec143-preview.FVsC8y`, environnement `34f5965e-48b1-408c-bcf8-2ae9c29c04d8`. Le rechargement du serveur clone charge seulement les fixtures, pas l'application active. Pas de fournisseur actif ou E2E de livraison. État in_progress unitaire couvert, pas rendu visuel dans ces fixtures historiques.

<a id="extrait-4"></a>

## Extrait 4

- T011 : code/tests gelés392 PASS (283 logique/109 UI),46 cas ajoutés au socle346. Revue interne finale APPROVE, aucun résidu ; relecteur rejoue392 en6,08s à11:32:20 CEST. Principal rejoue392 en5,40s à11:32:57 et reconfirme les quatre hashes. T011 cochée sur ces preuves, pas sur l'ancien audit A.
- T012 : format PASS1,361s ; lint0erreur/22warnings baseline ; types PASS ; build PASS49,45s/6141modules, warning chunk existant ; diff PASS. T012 cochée.
- Recette finale US4 : aperçu natif isolé à11:33–11:35, click ouvre, Enter ferme, Space ouvre. Après deux RAF, focus exact sur le même toggle et `aria-expanded` cohérent. Trois relais repliés à gauche sans fond/bordure, hauteur32px ; noms attestés/UUID courts, deux notes visibles ; ordinary et streaming natifs. Lien « Voir » conservé vers `https://example.com/`. Citation DOM réelle : sélection « Quote » dans la note, start120/end125, préfixe `r toi : Quote Résumé pour toi : ` ; résolution sur la dernière note visible, corps agent fermé CSS none. Navigation citation E2E non exécutée.
- Largeur : conteneur320px, toggle233,65625×32px, zéro débordement, style temporaire restauré. Ce n'est pas un viewport mobile. Resize demandé1280 non appliqué ; viewport réel2212×1382. Aucun clipboard hôte E2E ou livraison fournisseur revendiqué. Déconnexion fugace de l'aperçu résolue par `preview_open`, aucun restart T3.
- Captures inspectées visuellement par le principal, copiées dans la validation sans modifier les originaux ; recette distincte dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/ui-recipe-US4.json`.
- Arrêt à11:35:22 CEST : PIDs83544/83608 identifiés propres/nonFirefox, Ctrl-C des sessions41681/61589, exit130 normal. PIDs absents ensuite, ports5876/13916 sans listener. T3 actif PID85017, démarrage07:21:41 inchangé. Fixtures/tempdir/onglet masqué conservés, aucune suppression ou fermeture d'onglet revendiquée. Clone SQLite quick_check ok, projection_thread_sessions0 et provider_session_runtime0. Root Bridget142 conserve ses six items dirty.

<a id="extrait-5"></a>

## Extrait 5

Après réception du verdict, phase documentaire distincte : T013 cochée et statut Implemented,13/13, non installé/non activé. Ce changement de tâches n'est pas présenté comme une modification pendant la comparaison. Aucun second passage artificiel revendiqué. Durée du complément jusqu'au verdict :11:05:59→11:39:54,33min55s (2035s), hors finition documentaire. Les preuves historiques346/audit A sont conservées. Aucun commit, merge, push, installation, activation ou restart T3.

<a id="extrait-6"></a>

## Extrait 6

L'utilisateur autorise ensuite commit, fusion, push, installation et redémarrage T3 pour143. Nouvelle phase distincte ; les restrictions et résultats « non livré » ci-dessus décrivent l'histoire avant cette autorisation, pas une interdiction actuelle.

<a id="extrait-7"></a>

## Extrait 7

Faits reçus du principal au début de livraison : commit T3 `a1a4f2ef12` sur les quatre fichiers, fusion fast-forward `local/v0.0.45` effectuée, quatre hashes gelés392 inchangés. Push du fork en cours, résultat encore attendu. Installation et activation non prouvées à ce stade. L'agent documentaire ne lance ni build, daemon, boucle ni restart et ne touche pas142.

<a id="extrait-8"></a>

## Extrait 8

Faits suivants transmis par le principal : push réussi des branches T3 `local/v0.0.45` et `session-143-bridget-discreet`, commit complet `a1a4f2ef12e2e4c4f2b64cd6d4ac162bceb22c39`, vers `https://github.com/guthubrx/t3code.git`. Correctif capitalisé sur `local-patch/bridget-headings-v0.0.45` déjà choisie par le manifeste futur : cherry-pick `b8ffa6f2806fe94b7bba8c15e97ab4f674aac2ec` poussé, diff des quatre fichiers vide. Rejeu392 PASS en8,15s à12:07:12. Paquet complet construit223s/exit0 avec SHA consigné dans delivery/results ; sauvegarde application effectuée et snapshot SQLite readonly créé exit0,12,5GB. Son quick_check reste en cours. Candidat extraction/signature en cours, superviseur préparé non activé, aucun succès d'installation/restart annoncé.
