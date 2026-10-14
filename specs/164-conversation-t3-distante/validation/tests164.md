# Stratégie ciblée164 — Preuves attendues

**État actualisé :** Implemented,22/22 ; pipeline Livré avec G8/G9 acceptés. PreuvesR6, historiquesR4/R1, recetteweb et paquet qualifiés ci-dessous. Aucun nouveau test ni succès global déduit.

| Scénario | Oracle utilisateur | Niveau prévu |
|---|---|---|
| S164-01 — fragments_exact_unicode | Fragments compatibles → texte exact unique ; espaces, UTF8, lignes et Markdown conservés | Projection partagée |
| S164-02 — frontieres_message_session | Identités explicites incompatibles/session différente/nontexte séparent ; absence d'ID ne crée pas de rôle fournisseur | Projection partagée |
| S164-03 — gap_page_eviction | Deux gaps distincts, gap sticky répété, pression à 64 lacunes, éviction réducteur et headcut local ; page initiale/extrait et saut seq filtré sans faux gap | Projection et composants |
| S164-04 — replay_conflit | Rejeu identique sans doublon ; seq conflictuelle consultable sans écrasement silencieux | Projection et conteneur existant |
| S164-05 — outil_preuves | Started sans item_id distinct du completed avec résultat ; Claude appel seul, ACP corrélation explicite, refus Bridget neutre ; statut uniquement sur preuve | Projection et composants |
| S164-06 — inconnu_consultable | Forme inconnue/malformée conservée dans détails sûrs et accessible avec séquences | Projection et composants |
| S164-07 — markdown_sans_autorite | Prop inerte : `/etc/passwd`, image locale ou HTTPS et `<img onerror>` sans filesystem, chargement média ou action shell ; usages ordinaires du Markdown inchangés ; sélection/copie | Composants concernés |
| S164-08 — resultat_page | Résultat identifié séparément, incomplet jusqu'au dernier curseur ; copie du contenu chargé exacte | Tests Journal149 étendus |
| S164-09 — actions_statuts | En cours/result_available/dernier état connu distincts ; arrêt soumis scope/refus ; lien parent/reconnexion conservés | Tests Journal149 étendus |
| S164-10 — lifecycle_borne | Pause background/hidden/focus : abonnement et lectures annulés ; reprise cursor ; fenêtre de 4 096 événements, historique100 | Tests Journal149 réemployés/étendus |
| S164-11 — parite_clients | Même corpus donne mêmes blocs ; web/mobile adaptés aux primitives natives ; lecture seule intacte | Projection commune et deux composants |
| S164-12 — recette_visuelle | Vue cohérente, lisible sur largeur desktop/mobile, activité repliable et détail consultable ; aucune conversation/lancement neuf | ROOT client isolé, données synthétiques |

## Campagne

1. Avant code, inspecter seulement les formes de journal nécessaires. Réduire toute copie de données réelles et vérifier l'absence de secrets avant une revue externe. Privilégier fixtures synthétiques des formes observées.
2. Une campagne finale des trois groupes ciblés ; avant cela seuls retours courts nécessaires à l'implémentation. Après correction, rejouer le groupe affecté, pas les suites inchangées.
3. TypeScript/format/lint du périmètre modifié selon scripts du dépôt, Git diffcheck et self-review. Les contrôles Mac autorisés par ROOT peuvent compléter une toolchain Linux absente. Ne pas utiliser des binairesMac sur Linux.
4. ROOT inspecte un client isolé après intégration. Aucun sous-agent ne lance serveur/browser. La demande UI autorise cette vérification réversible. T3 vivant et ses données restent protégés.
5. Contre-revue code et audit : bornes, confidentialité, lecture seule, complexité, réemploi, surfaces et aucune dépendance nouvelle.

Commandes prévues :

```text
vp test run packages/client-runtime/src/bridgetJournalPresentation.test.ts packages/client-runtime/src/state/orchestration.bridget149.test.ts
vp test run apps/web/src/components/BridgetTaskJournal149.test.tsx apps/web/src/components/ChatMarkdown.permissions.test.tsx
vp test run apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx
```

Pas de contrôle repo-wide, pas de benchmark neuf, pas de suite Rust pour un diff164 sans Rust. Une recette web ne prouve pas le client natif mobile. Si le client natif n'est pas exercé, conserver cette limite dans le reçu, sans inventer un PASS. Une preuve de composant ne vaut pas une capture du paquet installé. Les résultats 163 ne valident pas les nouvelles sources 164.

## Coût du rendu à vérifier sans nouveau banc

Les oracles de composant vérifient qu’un ajout de queue conserve les références des blocs clos et ne remonte pas leur rendu. Le détail brut est absent tant qu’il reste fermé. Les scénarios de lacune incluent deux marqueurs encore entre des fragments chargés, puis plus de 64 frontières pour prouver une coupe conservative plutôt qu’un oubli. L’analyse de complexité porte sur le code de fusion/projection, pas sur un chiffre de débit inventé.

## Résultats observés R1 et suites nécessaires — historique au checkpoint R1
### Campagne ciblée exécutée

| Groupe | Résultat | Début UTC | Durée de la commande | Preuves locales privées |
|---|---|---|---:|---|
| runtime | 16 PASS, RC0 | 2026-10-13T22:23:43.786560+00:00 | 1.416s | /Users/moi/.cache/bridget-delivery164-20261013/tests/runtime164-r1.json ; /Users/moi/.cache/bridget-delivery164-20261013/tests/runtime164-r1.log |
| web | 21 PASS, RC0 | 2026-10-13T22:24:14.772732+00:00 | 2.367s | /Users/moi/.cache/bridget-delivery164-20261013/tests/web164-r1.json ; /Users/moi/.cache/bridget-delivery164-20261013/tests/web164-r1.log |
| mobile | 3 PASS, RC0 | 2026-10-13T22:24:37.495710+00:00 | 1.416s | /Users/moi/.cache/bridget-delivery164-20261013/tests/mobile164-r1.json ; /Users/moi/.cache/bridget-delivery164-20261013/tests/mobile164-r1.log |

Les groupes utilisent nice10 et `--maxWorkers 2`. Chaque reçu montre les hashes identiques avant/après son exécution. Runtime =6 tests de projection+10 du réducteur149. Web =13 tests Journal149+8 Markdown/permissions. Mobile =3 tests du composant149 dans son environnement de test ; ils ne prouvent pas une coque native iOS/Android.

Les logs web signalent la dépréciation de react-test-renderer et les refus d’actions attendus par les tests. Le journal mobile contient les avertissements act(), composants RN représentés dans le DOM et props natives non reconnues. Ces avertissements sont conservés : aucune absence d’erreur visuelle native n’en est déduite.

### Contrôles complémentaires et limites

| Contrôle | Résultat réel | Limite restante |
|---|---|---|
| Vérification initiale du format sur11 chemins | RC1, neuf fichiers à formater | Ce contrôle initial reste historique |
| Format appliqué sur les neuf fichiers | RC0 ; log99ms, commande252ms | Aucun recheck du format ni rejeu des tests après formatage au checkpoint |
| Garde AST normalisée | RC1 ; six fichiers égaux, trois différents ; zéro erreur parseur avant/après | Caractériser les trois différences avant copie des sources ; ne pas les qualifier de simples espaces sans preuve |
| Lint du périmètre | RC1, chargement configuration bloqué | Plugin workspace @oxlint/plugins non exposé à oxlint-plugin-t3code ; ce résultat n’est pas un lint du code achevé |
| TypeScript client-runtime | RC1, une erreurTS2322 de fixture à la ligne22 : unknown n’est pas Json ; suggestions Effect également présentes | Annotation de payload corrigée en type wire BridgetJournalEvent["payload"] selon ROOT ; vingt charges de fixture validées JSON par gardeRC0, mais TypeScript non rejoué |
| TypeScript web | RC1, dix diagnostics | Log entier byte-identique au log163r1 ; correction des deux fixtures protégées déjà prouvée par163r2. Aucun PASS TypeScript164 intégré n’en découle |
| TypeScript mobile | RC0, log vide,6.676s | Sur snapshot contrôlé seulement ; ne prouve pas thème, sélection et rendu natif réel |

La garde AST signale :
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.ts
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal.tsx
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx

Le garde-fou compare le formatage à l’annotation de type déjà autorisée. Il ne fournit pas encore une preuve d’équivalence sémantique sur ces trois fichiers. Les neuf fichiers formatés restent distants ; la copie Mac est différée selon l’attestation. Le formatter rapporte16 threads dans son log ; les tests disposent bien de maxWorkers2. Aucun gain de performance n’est affirmé.

Provenance des diagnostics web :

- /Users/moi/.cache/bridget-delivery163-20261013/t3-typecheck-web163-r1.log — SHA25609b78c3a62f183902acc04b49f086fac4f3ba5fafefa7cb26290a6c3f9c5845e ; identique byte par byte à types-web164-r1.log.
- /Users/moi/.cache/bridget-delivery163-20261013/t3-fixtures163-r2.json — SHA256ce697573dd3866f5e736e847c97e7cdce4e56d79d570c3814029cd6b908c5a36 ; typecheck web163r2 RC0 après correction de providerThreadId:null dans les deux fixtures MessagesTimeline. Ce reçu ne valide pas les nouvelles sources164.
- /Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-attestation.json — autorité des copies et limites. Les fixtures MessagesTimeline protégées ne doivent pas être modifiées par164.

### Deux écarts fonctionnels encore actifs

Constats de lecture ROOT, pas échecs prouvés par les quarante tests :

1. **FR-015/T010 — Thème et sélection mobile.** Le Markdown Nitro courant ne transmet pas encore toutes les préférences de thème/police/tailles et la sélection des paragraphes/titres/code-inline. Réemploi des primitives et préférences T3 demandé ; aucune réduction au texte brut.
2. **FR-005/T005 — Faux extrait à une frontière normale.** Un changement de session ou de livraison doit séparer deux blocs sans annoncer que le début manque. Les vrais headcuts/pages/gaps conservent l’indication d’extrait. Aucun gap déduit d’un saut de séquence filtré.

La mission de production3182cc37-7204-4769-9439-644b813a23c1 est revenue stable selon ROOT. Nouvelle mission corrective : **3b3905be-3214-44bd-b160-37ab4e5edc9f**, enfant **2727d824-1353-4f8a-9d02-f472659e6841**. Son reçu initial est starting/working ; consigne de lecture seule avant GO après caractérisation du formatage. Aucun correctif accompli ni résultat de test n’est présumé.

Propriété limitée à quatre fichiers sous la copie Cartae :
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.ts
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.test.ts
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx

Reçu de la mission corrective : /Users/moi/.cache/bridget-delivery164-20261013/delegation-corrections164-r2.json. ROOT conserve les contrôles et le permis compilateur ; aucun test/build permis au correcteur à ce stade.

**Converge non terminé ; contre-revue/audit code et recette visuelle non validés ; aucune livraison, activation, push ou nettoyage164 attesté. Aucun cochage nouveau.** ETA02:52–04:52CEST conservée, hors quota et attente163. Les gates restantes ne sont pas déduites de la campagne ciblée.

## Checkpoint actualisé — R3 puis reprise R4

**In Progress, 2/20. Aucun cochage nouveau.** Les checkpoint R1 et le reçu initial du correcteur ci-dessus restent historiques. Les résultats qui suivent concernent leurs snapshots précis ; aucune somme R1+R3+R4 n’est présentée comme campagne finale.

### Sources et contrôles R3

- `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r3-final.json` — SHA-256 `a6546dd14b6135ed58374b6282105fe906ce01ce8eca2a9f7fcba6dd3314be56` : douze sources copiées sur Mac, dont vite.config.ts ; hashes relus et identiques lors de la self-review R3.
- `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r3-summary.json` — SHA-256 `0ff697bb5c6431a390b0c373b32974fcdbca7d610ee00cca9f211f1ba2484424`.
- `/Users/moi/.cache/bridget-delivery164-20261013/tests/affected-test-results164-r3.json` — SHA-256 `66d3afd06334d20b1a23e7011cda7f1a34d4a8bbeb4f623b1f304012b3ef615f`.

| Contrôle R3 | Preuve réelle | Limite |
|---|---|---|
| Tests affectés | 24 PASS/0 FAIL : projection7, web13, mobile4 | Réducteur10 et Markdown8 sont les preuves R1 conservées, pas une nouvelle campagne de42 tests |
| Format final et garde sémantique | RC0 après formatage ciblé | Les échecs initiaux restent historiques |
| Lint | RC0, 14 avertissements/0 erreur | Les avertissements restent visibles |
| TypeScript runtime et mobile | RC0/RC0 | Aucun TypeScript web R3 |
| Thème/sélection mobile et frontière normale | Correctifs R3 relus ; oracles « rend titres, code et liens dans le lecteur sélectionnable aux préférences T3 » et « distingue les frontières de livraison et session des extraits réellement coupés » PASS | Tests avec fixtures/stubs ; aucun appareil natif exercé |

La liste interop de `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts` ajoute uniquement le conteneur mobile, car les props Markdown natives ne consomment pas `className`. La règle garde `error` avec `allowUniwindTheme: true` dans la liste existante. Aucun changement global de règle.

### Retour code R4 : correction annoncée, sans contrôle exécuté par le développeur

Reçu `/Users/moi/.cache/bridget-delivery164-20261013/retour-cartae164-r4.json` — SHA-256 `8afea0a99fcff31f4cacc23e04b799a8cc9813c8379ad29366164dcbce0cf8c6`. Mission `3b3905be-3214-44bd-b160-37ab4e5edc9f`, enfant `2727d824-1353-4f8a-9d02-f472659e6841`, ronde4, retour `result_available`. Le développeur n’a exécuté ni test, lint, TypeScript, formatage ni build.

L’écart R3 concerne FR-005/FR-011/FR-012 : une nouvelle visite pouvait oublier G1 ou la coupe conservative alors que ses événements restaient chargés. Le retour R4 annonce leur fusion atomique avec les événements dans le même `setJournal`, sur web et mobile. Un snapshot G2 seul, puis vide, conserve les frontières pertinentes. Le helper existant déduplique et prune seulement hors fenêtre. Bornes maintenues : 4 096 événements, 64 lacunes ; aucun gap déduit d’un saut numérique.

Les six fichiers du retour sont sous `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164` :

| Source distante complète | SHA-256 annoncé au retour, avant contrôles ROOT |
|---|---|
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.ts | `ac984775284ab50e4e2984fb9f7a3f744c8cfe117bf041270f1efe357fff256d` |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/packages/client-runtime/src/bridgetJournalPresentation.test.ts | `3861a60ea871895ec923809ca225d1a68ce789ba5a6eea22d61389cc23a47d24` |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal.tsx | `0a597e25ec87c353757472a3bcb08d38f3e14a63a57788dc93bc373168930f45` |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/web/src/components/BridgetTaskJournal149.test.tsx | `74914daa0c4496d638797a08582526c50e28fb246bc2873d191ed51470464b8d` |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal.tsx | `254d9e94bc7707f462f07a2a4ebf07422e26970d91bfcfbe614151c76b49419b` |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164/apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx | `368f4fcd3b5e97f4c682866ff4de971913e46c880e199763ef55841b8f6329b2` |

Ces empreintes sont la provenance du retour, pas les empreintes finales après formatage ni une preuve de copie Mac R4.

### Contrôles ROOT R4 reçus ensuite

- `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r4-summary.json` — SHA-256 `35eae7160f7379c869425d280ae3aad4f127543258967b4cd61158a9beec84b0`.
- `/Users/moi/.cache/bridget-delivery164-20261013/tests/affected-test-results164-r4.json` — SHA-256 `ec47ab2b320324ecb5308fdd762c6b89a4b323c6ee2e0821cca0629217b46489`.

| Contrôle R4 | Résultat | Limite |
|---|---|---|
| Quatre fichiers de tests ciblés | 36 PASS/0 FAIL : projection7, réducteur10, web14, mobile5 | Markdown8 R1 non rejoués ; pas de GREEN global |
| Reprise web/mobile | « conserve les lacunes et la coupe chargées après une nouvelle visite partielle » et « conserve les lacunes et la coupe chargées après une reprise mobile partielle » PASS | Relecture du code Mac R4 après transfert stable encore attendue |
| Format final | RC0 | Vérification initiale RC1 conservée |
| Garde AST stricte | RC1, égalité stricte fausse | Ne pas la réécrire en PASS |
| Qualification sémantique Babel7.29.7 | RC0 : expressions et enfants JSX rendus équivalents après retrait des positions/commentaires et ParenthesizedExpression | Qualification distincte, pas égalité AST stricte |
| Lint | RC0, 15 avertissements/0 erreur | Avertissements conservés |
| TypeScript runtime/mobile | RC0/RC0 | Aucun TypeScript web R4 |

Les reçus attestent les métadonnées des54 dépendances/toolchain inchangées. Les logs gardent les avertissements react-test-renderer/act et tags natifs représentés dans le DOM. Les tests ne prouvent ni rendu sur appareil réel, ni recette Electron, ni paquet installé. Aucun contrôle n’a été relancé par le responsable documentaire.

### Gates restantes

Au checkpoint documentaire, le Mac porte encore R3. Attendre le transfert R4 via delivery_helpers163 et le GO ROOT pour la relecture en lecture seule des six sources et oracles, avec stabilité des douze empreintes avant/après. La fermeture du finding ne doit pas précéder cette lecture.

TypeScript web/T012 attend le rapprochement des deux fixtures163 protégées : les dix diagnostics historiques sont corrigés et prouvés dans163, sans PASS web164 intégré. Les gates source163 validée/intégrée/poussée restent bloquantes. Converge164 formel, contre-revue/audit, S164-12/T017, paquet, installation, push et nettoyage restent à attester. Aucun résultat futur ni nouvelle case cochée. ETA documentaire02:52–04:52CEST conservée, hors quota et attente163 ; ROOT la recale selon les gates.

## Checkpoint Converge P1 et TypeScript web final

TypeScript web exécuté une seule fois par ROOT : RC0,12,8336s, log vide, source12/métadonnées54 inchangées. Reçu `/Users/moi/.cache/bridget-delivery164-20261013/tests/types-web164-final.json`, SHA-256 `7e3bf25a886c286d775b58832ac9f1cdd767e12747552c96b5d06708a6257b3a`. Les deux fixtures163 corrigées sont rapprochées, sans source164 changée. Les attentes de copie Mac et TypeScript web des checkpoint précédents sont historiques : transfert R4 vérifié stable, finding reconnexion fermé.

**Converge passage1 NON CONVERGED, T021 ajoutée, In Progress2/21.** Les détails originaux et témoins masquent des métadonnées v/ts sur web/mobile ; les36PASS existants ne couvrent pas cette ouverture. Une correction ciblée et les deux bancs Journal149 existants sont requis. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p1.md`. Aucun contrôle relancé par le responsable documentaire ni source modifiée. Contre-revue/audit, S164-12/T017, Git/paquet/install/cleanup/runtime restent futurs. Aucune case cochée.

## Checkpoint P2 — convergence source R5b

**CONVERGED sur source, zéro tâche ajoutée. In Progress2/21 cases acceptées au checkpoint.** C164-01/T021 fermé par enveloppes originales et témoins entiers, rendu seulement après ouverture, oracles web/mobile v1 conformes et ts distincts. Passage1 NON CONVERGED conservé. ROOT coche après acceptation ; aucune case modifiée ici.

Source12 Mac/snapshot/remote stable selon `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r5b-precheck.json` SHA-256 `27e132d5f6ab1c2cdf2e362319b04314032c56991968d3960188c9b6f95f2228`. Deux fixtures MessagesTimeline héritées163 restent distinctes des douze sources164 et protégées.

Contrôles `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r5b-summary.json` SHA-256 `3c09f3ec97c0e5eeb3417481d5e3f23ec37769c9a14c307d68ecddd8b036f1f6` :21PASS/0FAIL(15web6mobile), format4/AST stricte/qualification Babel RC0, lint4 RC0 avec8avertissements/0erreur, TypeScript web1,8040s/mobile3,0063s RC0. Les sources/métadonnées54/deux fixtures163 restent inchangées pendant les contrôles. Aucun replay des36, du helper/réducteur/runtimeTC ou Markdown inchangé. Ne pas présenter les preuves conservées comme une nouvelle suite totale.

Rapport `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p2.md`. Prochaine gate T014 contre-revue externe/audit, puis T015–T020 source163/Git/paquet/recette/install/cleanup. Recette web/mobile/native, livraison et runtime restent futurs. Aucun test/service/Git/code relancé ou modifié par cette lecture documentaire.

## Checkpoint revue R1 — R6 corrective ouverte

Contre-revue R5b APPROVE_WITH_CHANGES, SHA du rapport `0c16373f4f8b253c4f2afc39ea7bee88eb2dc59f2b965f4c1c3bdd8766ac5b73`. ROOT confirme F1 MEDIUM dictionnaire hérité, F2 LOW replay du témoin et F3 LOW préfixe explicitement coupé. T022 ouverte, progression2/22 ; F4 huit warnings React non bloquants. C164-01/T021 fermés et P1/P2 historiques conservés byte-identiques.

Mission corrective `3b3905be-3214-44bd-b160-37ab4e5edc9f`, ronde6, six fichiers déjà dans le scope. Code seulement, sans contrôles du développeur tant que163R8 réserve le permis. Aucun retour stable, hash final, test, P3 ou auditR6 présumé. Trois bancs existants affectés après permis ROOT ; pas de suite globale ou outil nouveau. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r1.md`. T014 complète, recette et livraison restent ouvertes ; aucune case cochée.

## Mise à jour opérationnelle — remplacement du producteur R6

La reprise R6 de la mission `3b3905be-3214-44bd-b160-37ab4e5edc9f` a échoué avant édition : enfant `2727d824-1353-4f8a-9d02-f472659e6841` en DND,31min restantes. Preuve : `/Users/moi/.cache/bridget-delivery164-20261013/native164-r6-dnd-failure.json`, SHA-256 `2938e3c2bedf2165000c7b409cfc6aeedd087f821122bf6662756da85914df0b`. ROOT a demandé cancel ; état cancelling communiqué, fin non encore prouvée. Aucun DND contourné ni droit modifié.

Producteur de remplacement admis par ROOT : mission `c5ce2e7a-f726-44c4-8b87-bcc94e926005`, enfant `2fb66854-3a0e-47e2-839f-2e2bc1b0d695`, request `164-review-corrections-r6-replacement-20261014`. Même correctif R6/T022 et même propriété six fichiers UI/helper/oracles, code seulement, aucun contrôle permis. Ce remplacement ne crée ni nouvelle tâche ni nouveau Converge. Source R6, retour stable, tests et correction validée restent non attestés ; P1/P2 et R5b conservés.

## Checkpoint R6 postformat/P3 — source corrigée et prouvée

R6 final Cartae direct, transféré Mac et stable14empreintes. Manifest `/Users/moi/.cache/bridget-delivery164-20261013/tests/validation164-r6-final.json` SHA-256 `10ba64b098085bb53cef6669f53d02bae7919fa86482d03a99f1b19aa9efa89c`. Source corrective directe selon source164-r6-code.json ; missionc5ce2e7a-f726-44c4-8b87-bcc94e926005 cancelled, fournisseur natif non auteur des edits. Les anciennes admissions/DND restent historiques.

**ConvergeP3CONVERGED sursource, aucune nouvelle tâche.** F1/F2/F3 etT022 corrigés/prouvés ; C164-01/T021 conservé fermé.28PASS7helper/15web/6mobile, TypeScriptruntime/web/mobileRC0, format6/ASTstricte/lint6RC0 ; Babelnonexécuté.9warnings/0erreur, huitReactinchangés et no-accumulating-spread helper déjàprésentR4. Leshashespostformat tests/TC3restentstables et correspondentau transfert ; aucuncontrôle relancé par DOC164.

Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md`. P1/P2 ettasks byte-identiques pendantP3 ; casesROOTtoujours2/22. Contre-revueR2/deltaaudit/T014complète puis Git/QA/paquet/installG8G9 futurs. Aucun succès produit/runtime déduit.

## Checkpoint final R6 — P3 et revue indépendante R2

**P3 CONVERGED sur la source ; R2 APPROVE.** F1/F2/F3 sont fermées. Le compteur représente des occurrences, pas des versions uniques. F4 est accepté et non relu. Rapport et limites : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`.

Audit incrémental de six sources,309lignes lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01`. GradeA100 ciblé,0finding actif ; validateur canonique RC0,0erreur/0warning. AuditR5b, P1/P2 et tasks conservés byte-identiques. Aucun nouveau test ni contrôle source relancé. Les28PASS et trois TypeScript RC0 proviennent du manifeste finalR6, pas de la lecture documentaire.

ROOT reste responsable de l'acceptation des cases. Progression documentaire2/22 inchangée. T015–T020, recette UI/native, Git/paquet/installation/cleanup/G8/G9 restent futurs. Aucun succès produit ou runtime présumé.

## Recette intégrée S164-12 acceptée — T017, 2026-10-14

ROOT accepte la recette du serveur et du client web réellement construits sur source511190beb307. Le daemon est remplacé par la fixture CLI synthétique existante. Le serveur isolé utilise son propre dossier de données ; aucune base active T3 ni session utilisateur n'est modifiée. Le reçu final ci-dessous complète l'observation historique de la page suivante ; cette première observation incomplète reste conservée.

| Observation réelle | Preuve et limite |
|---|---|
| Texte/Markdown lisibles ; frontières normales sans faux extrait | Capture web sur paquet final ; activité initialement repliée. |
| Commande achevée et détail inconnu | Sortie/exit/statut visibles ; enveloppe inconnue seq12 entière ; HTML littéral, aucune image ni script exécuté ; aucun composeur. |
| Deux lacunes successives |126–127 reprise128, puis129–130 reprise131 ; alertes conservées, retour au suivi/reconnexion131. |
| Historique paginé |125événements initiaux ; première page100 ; pages suivantes101–128 puis131 observées dans le supplément final. hasNext est un comportement de fixture, pas une preuve de tout l'historique. |
| Résultat durable |28070octets totaux ; fenêtre initiale partielle puis suite ; paragraphes27999/36caractères et fin exacte « Fin du resultat durable synthetique » ; plus d'indication partielle ni bouton Suite. |
| Présentation sur deux tailles |1280×900 et390×844 ; capture web responsive seulement. Pas de pixels React Native ni redémarrage Electron. |
| Fermeture de la recette | Serveur privé68778 identifié puis SIGTERM unique, absent après3s ; deux sessions privées révoquées, zéro restante. Pas de terminaison étrangère. |

Reçu final : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/t017164-final.json`, SHA-256 `fcf03a0e7f86ca30b13497df82a0fd9b512232e5db516eb0bd11f22fb9d47f6f`. Sept captures privées byte-identiques et archive66membres rouverts/hash vérifiés : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz`, SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. Archive exclut credentials et DB privés. Les erreurs initiales RPC400 et erreurs d'observation sont conservées ; correction limitée aux scripts de recette privés. Aucun source de production modifié pour cette recette.

Les28PASS de sourceR6 restent une campagne distincte. Contrôles du paquet : R1 historique2574PASS/2FAIL ZIP ; correction de réarchivage seulement ; R2 exactement2oracles ZIP PASS. Aucun nouveau build, suiteFULL, dépendance ou modèle pendant S164-12. Aucun succès global2576 déduit.

Progression actuelle19/22. T018/G8, T019/G9 et T020 restent non attestées. Signature/installées sur disque ne valent pas code activé dans PID704/969.

## Acceptation ROOT — G8/G9 attestés, 2026-10-14

**Progression actuelle : In Progress,21/22.** ROOT accepte G8 et G9 sur les reçus réels. T018/T019 sont cochées. T020 reste ouverte jusqu’au commit/push documentaire final par ROOT. Aucun push futur n’est attesté.

G8 opérationnel réel accepté sur source Bridget `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` : `/Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-final.json`, SHA-256 `5b1c00923f2f307136b5831cc8d3328227b7284deacf44c793d71542e3ceb52c`. Résultat durable `RECETTE163_G8_OK`, un reçu ; OFF explicite prouvé puis replay ON de la même clé inchangé. Le replay ne constitue ni nouvelle tâche ni nouvel appel de décision. La recette HTTP de décision reste celle du laboratoire T040R4. Le provider réel prouvé par cette recette est codex-pro ; autres profils configurés sans nouvelle attestation quota/auth. ROOT atteste daemon31298/pont31420 et workerCartae1454489 actifs, trois routes autorisées Bridget/Roadie/Opus2D. Aucune inspection des capacités en mémoire ni mission de modèle distant supplémentaire déduite.

T3 : source `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée vers le fork/local/main-20261009. Application signée `/Applications/T3 Code (Local).app` installée en0.0.45-local.164 ; PID704/969 et naissances inchangés. Code162 encore chargé, aucun redémarrage ni activation différée. Bridget demeure indépendant de T3.

Nettoyage T3 WT164 et branche attesté : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt164-fresh-stable-retirement.json`, SHA-256 `b0661c19c7243969d90035dfd8a9de062e2a823e5ae2183c78ac8fa6553f6b1b` ; retrait sans force après source intégrée/poussée et garde lsof complète,321991571octets logiques. Les anciens chemins WT164 sont historiques. G9 final : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json`, SHA-256 `08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3`. Onze ressources retirées : trois worktrees, deux stages Mac et six namespaces TMP ; trois branches retirées. Le WT T3 de163 et sa branche sont conservés : inventaire complet282139fichiers réussi, puis inventaire exact changé après archive. La garde refuse le retrait ; aucune cause de changement inventée, aucun contournement.

Conservations Cartae justifiées : `/Users/moi/.cache/bridget-delivery163-20261013/g9-cartae-final-usage-retentions.json`, SHA-256 `cb23362780df36d938f321c054c0fe626f0021194d65dbb78ed30f2e8852d359`. lsof RC0 mais incomplet à cause de FUSEportal `/run/user/1002/doc` ; aucun contournement. Sont conservés `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission`, sa copie `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164`, `/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target` et les cinq namespaces `/home/moi/.local/state/bridget-delivery164/tests/tmp-r2`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r3`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r4`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r5b`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r6`. Pas de retrait distant ni de gain physique revendiqué. Rollback, preuves, DB et travaux étrangers restent protégés.

S164-12 est une recette web réelle du paquet, avec données synthétiques : sept captures et archive66membres SHA vérifiés, archive `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz` SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. La taille390×844 prouve le web responsive, pas React Native natif ni Electron actif.

Tests sources :28PASSR6 distincts des10réducteurR4 et8MarkdownR1 conservés, soit46preuves de tests qualifiées par provenance, pas une nouvelle campagne46 ni un secondFULL. L'échec historique de rejeu/réalignement reste historique ; sa correction est couverte par l'oracleR6, aucun replay supplémentaire déduit de ce texte. Paquet :2574PASS/2ZIPFAILR1 conservés puis deux seuls oracles ZIP PASSR2 après réarchivage ; pas de nouveauGREEN2576. P1/P2/P3, contre-revues et audits ciblés restent inchangés ; aucun succès global/runtime de T3 inventé.

### Conservations G9 supplémentaires

Les caches `/Users/moi/.cache/t3-final162.2nosz6zd` et `/Users/moi/.cache/t3-final163.zsb7fmfu` sont conservés pour leurs bases SQLite persistantes réelles,20480octets chacune, aux chemins `/Users/moi/.cache/t3-final162.2nosz6zd/source/node_modules/.vite/task-cache/v18/cache.db` et `/Users/moi/.cache/t3-final163.zsb7fmfu/source/node_modules/.vite/task-cache/v18/cache.db`. Ces dépendances ne sont pas conservées sous prétexte d'un futur build163. L'ancien échange `/Applications/.T3 Code (Local).app.new-163` reste utilisé par les processus T3 protégés ; le rollback et les preuves restent conservés. Cache Cargo partagé `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug` conservé, permis lourd rendu ; aucun nouveau FULL ou build163 en attente prétendu. Les trois namespaces SQLite `/tmp/b163.uqd3443p`, `/tmp/b163.dxmfq_la` et `/tmp/b163.07di472q` sont protégés par leurs attestations historiques ; leur existence actuelle n'est pas revendiquée. Aucun gain physique ni modification DB de production revendiqué.

## Clôture documentaire acceptée par ROOT — 2026-10-14

**Statut canonique : Implemented. Progression :22/22. Pipeline : Livré, G8/G9 acceptés avec conservations justifiées.** T020 est cochée sur le push documentaire réel `c57cfd35efa1338cda5256a02c57d71258d8b8d4` vers github/main. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json`, SHA-256 `56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390` ; HEAD et référence distante égaux, racine propre à cette observation. Ce commit contient les documents de livraison G8/G9 avant le cochage final. Les cases fermées sont intégrées et poussées dans `af8998a7d7494913d4420e2dca7354a73bd8a4f5`, vérifié par ROOT sur github/main égal à HEAD et racine propre. Cartae main et worktree sont alignés sur ce commit ; archive25membres vérifiée byte-identique. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json`. Les checkpoints21/22 précédents restent historiques.

Le build actif Bridget est toujours source `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` ; le commit de métadonnéesc57 est distinct et ne prétend pas une nouvelle compilation. Source T3 `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée au fork/local/main-20261009, application164 signée sur disque. T3PID704/969 reste chargé en162, sans redémarrage ni activation différée.

La recette S164-12 prouve le client web construit avec données synthétiques, en taille desktop et mobileCSS ; elle ne vaut pas Electron actif ni React Native natif. Les tests et les audits ciblés restent qualifiés par leur provenance et couverture partielle. Aucun nouveau Converge, audit, test, build ou succès global GREEN n'est créé par cette clôture. Bridget reste indépendant de T3.
