# Recette UI web RÉELLE native 149 - T040 / SC001 - ronde r1

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production, aucun modèle, aucun Cargo, aucune seconde app T3, aucun launchd.
Cette recette remplace la recette navigateur à daemon SIMULÉ (`t3-runtime-hardening-sonnet-r5.md`, `bridget_fixture.mjs`) par le VRAI daemon Bridget r8.

## Verdict : APPROVE sur le périmètre exécuté (UI web réelle + daemon réel + enfants fermés)

Hors périmètre : modèle réel (T037), coque desktop, mobile natif, iOS. Le modèle affiché (`fixture-model-149`, `fixture-model-149-mini`) est une donnée de fixture. Aucun modèle n'a tourné.

## Couches

| Couche | État |
|---|---|
| Daemon Bridget release r8 (SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6`, vérifié avant et après) | RÉEL |
| Serveur T3 `apps/server/src/bin.ts` (base privée, auth scopée, `BridgetReader`, `BridgetLineage`), UNE seule instance qui émet le credential MCP et sert Lineage (comme `unified149.ts`) | RÉEL |
| UI web du worktree T3 (Vite+, sources INCHANGÉES : `git diff HEAD` `59a54267ce7c1d2a` avant et après) ouverte dans la preview T3 | RÉEL |
| `delegate` -> enfants natifs lancés par le daemon | RÉEL ; enfants = faux serveur app-server Codex fermé (`codex149b.py`), explicite |
| Wrapper du fil T3 côté daemon (trames `Register`, `T3ThreadBindingFact`) | SIMULÉ (trames réelles émises par la recette) |
| Pair Codex de T3 | SIMULÉ (réponses capturées, tour tenu ouvert) |

Ports privés : serveur T3 15758, UI 15778. Fixture : `/Users/moi/.cache/bridget149-native-interop.w9UAwP` (0700). Script : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/ui-native149.ts`. Résultat : `.../results-r3/ui-native149.json`.

Ouverture : j'ai interrogé l'état du preview T3 d'abord (`preview_status`, `t3_preview_list` : aucun onglet, profils `default` et `incognito`). J'ai ouvert un onglet à moi, profil incognito, avec `preview_open`. Pas de navigateur de repli. Le lien d'appairage est un jeton jetable de la fixture, jamais écrit dans un rapport.

## Scénario et résultats

Trois racines admises par le credential émis par T3 : R1 (`fixture-model-149`) dont le descendant R1g diffuse des fragments en continu ; R2 (`fixture-model-149-mini`) qui se termine ; R3 (`fixture-model-149`) active, à arrêter depuis l'UI.

| Vérification | Résultat | Preuve |
|---|---|---|
| Panneau Lineage du fil parent : « Lineage · 1 running », deux agents actifs, « Previous agents (1) » | PASS | capture 01 |
| Racine R1 : statut « Attend ses enfants », journal réel (`turn_start`, `prompt_dispatched`, `update`), barre « Tâche Bridget · lecture seule » | PASS | capture 02 |
| Descendant R1g actif : journal en DIRECT. `chunkN;` : 85 -> 90 occurrences en 6 s (1 fragment toutes les 1,2 s) ; lien vers le parent « Waiting » | PASS | capture 03 |
| Zéro champ de saisie : 0 `textarea`, 0 `contenteditable`, 0 `form` (mesuré à 1280, enfant actif) ; pas de débordement horizontal à 1280 | PASS | mesure `preview_evaluate` |
| 375 px : modèle `fixture-model-149 · high`, statut « En cours », boutons « Ouvrir le parent », « Arrêter », « Reconnecter », barre lecture seule | PASS (visuel) | capture 04 |
| Terminal R2 : « fixture-model-149-mini · high », « Résultat disponible », résultat `chunk0;...chunk4;fixture149-answer:ui2` (375 et 1280), plus de bouton « Arrêter » | PASS | captures 05, 06 |
| Remise de R2 au parent : UNE (rapport de la fixture) | PASS | `report.json` de la fixture |
| « Arrêter » sur R3 (clic réel) : tâche `cancelled`, **PID 48686 réellement terminé**, les autres PID (R1 48642, R1g 48724) vivants, tâches R1/R1g inchangées | PASS | captures 07, 08 ; rapport avant/après |
| Aucun tour T3 : `turn/start` vus par le pair de T3 = 1 avant et après ; sessions fournisseur T3 = 1 ; aucune session pour les enfants | PASS | rapport de la fixture |
| Redémarrage du serveur T3 (SIGTERM vérifié) : l'UI se reconnecte, le journal de R1g se recharge (200 fragments), le daemon et les enfants Bridget ne bougent pas (PID vivants, tâches `working`/`waiting_for_children`) | PASS | capture 09 ; rapport |

Captures (privées, absolues) : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-native/` :
`01-fil-parent-panneau-lineage-1280.png`, `02-racine-attend-ses-enfants-journal-reel-1280.png`, `03-enfant-actif-journal-en-direct-parent-lien-1280.png`, `04-enfant-actif-375-modele-statut-boutons.png`, `05-terminal-resultat-modele-mini-375.png`, `06-terminal-resultat-1280.png`, `07-enfant-actif-avant-arret-1280.png`, `08-apres-arret-annule-1280.png`, `09-apres-redemarrage-t3-reconnecte-1280.png`.
Je n'ai pas refait tout le matériel de marges et de padding de la ronde r5 (déjà mesuré, composant inchangé). Je n'ai pas mesuré le débordement horizontal à 375 px : seulement une vérification visuelle.

## Observations (sans correction demandée)

1. À l'annulation d'un enfant, la page affiche « Bridget indisponible (journal_unavailable). Le contenu déjà lu est conservé. ». Bridget est pourtant disponible (le reste de la page se met à jour). Cause : le suivi de journal d'un enfant arrêté se termine par `journal_unavailable` (observé aussi à la CLI, `interop149.md` F6). Texte trompeur.
2. L'arrêt d'un enfant depuis l'UI envoie au parent une notice simple « Échec de livraison de la demande #... » (sans `in_reply_to`) en plus de l'annulation. Vue une fois (rapport de la fixture). Je n'ai pas isolé son origine.
3. Artefacts de la fixture, pas de Bridget : la sonde de version de T3 sur le faux `codex` affiche « Codex 0.0.0 is known to be broken » et « Update Available: Codex v0.162.1 ». La première bannière recouvre les boutons d'action à 1280 tant qu'on ne la ferme pas (mon premier clic sur « Arrêter » est tombé sur la bannière, sans effet, constaté par le rapport de la fixture). Je n'ai pas cliqué « Update ».
4. Après le redémarrage de T3, le run du fil parent est `cancelled` côté T3 (reprise propre de T3 sur un tour tenu ouvert). Sans lien avec Bridget : ses tâches n'ont pas bougé.
5. Les titres de tâches affichés sont le texte brut de la mission (`NONCE_ui1 NESTED_149[...]`) : c'est la donnée de la fixture.

## Nettoyage

Onglet preview fermé (`t3_preview_close`). Marqueur `finish` de la fixture : daemon arrêté par SIGTERM vérifié, serveur T3 arrêté, 0 fournisseur vivant. Reste constaté : le listener Vite (PID 48442, `ppid 1`, commande `vite-plus-core` dans le worktree T3, seul processus sur mon port 15778) a survécu au SIGTERM de son parent `vp` ; je l'ai arrêté seul (`kill 48442` sans `-9`, vérifié à 3 s, port libre). Jetons d'appairage et preuves de credentials supprimés de la fixture. Fixture conservée : `/Users/moi/.cache/bridget149-native-interop.w9UAwP`.

## Non prouvé

Modèle réel, mobile, desktop, MCP privé de bout en bout sous un vrai Codex, launchd. Le nouveau `wrapper.rs` modifié à 19:19 par un autre propriétaire n'est pas dans le binaire r8 testé.
