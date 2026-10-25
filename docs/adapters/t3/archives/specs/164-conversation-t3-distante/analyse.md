# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/analyse.md](../../../../../../specs/164-conversation-t3-distante/analyse.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Axe | Verdict documentaire | Conséquence |
|---|---|---|
| Indépendance Bridget | Cohérent | Aucun Rust/protocole/store/service ; présentation T3 seulement |
| Réemploi | Cohérent | Conteneurs, flux client, Markdown/primitives conservés ; opt-in inerte et gaps bornés explicitement autorisés |
| Fidélité | Cohérent | message_id est livraison, contenu exact, frontières explicites, formes started/completed distinctes |
| Continuité | Cohérent | Deux gaps, sticky, pression64, headcut local et page/extrait ont leurs tâches et oracles |
| Rejeu | Cohérent | Fusion avant Map, premier conservé, conflits visibles et bornés |
| Sécurité de présentation | Cohérent | Aucun accès distant via filesystem local, média auto ou action HTML/shell ; Markdown mobile maintenu |
| États/permissions | Cohérent | Métier distinct de caught_up ; actions natives et lecture seule conservées |
| Coût et ressources | Cohérent | Fenêtre4096/pages100, montage brut différé, blocs clos partagés/mémoïsés, O(n+c), aucun RPC par événement |
| Surfaces | Cohérent | Web/desktop et mobile ; renderer mobile adapté sans hooks/cartes actives du thread |
| Contrôles | Cohérent | Groupes ciblés, replays affectés seulement, pas de repo-wide ou suite Rust164 |
| Livraison | Cohérent | Source validée intégrée/poussée après163, paquet signé, recette isolée, install sans restart, G8/G9 et métadonnées finales |
| Parallélisme | Cohérent | SourceCartae temporaire distincte ; cwd163 conservé jusqu’à clôture164, aucune édition concurrente des neuf fichiers163 |

<a id="extrait-2"></a>

## Extrait 2

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/artefact-check164.json

<a id="extrait-3"></a>

## Extrait 3

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/spec.md — SHA-256 `912404ceab57031a1de152b5b2a6d762b8fd6978104071bb86c869e32717c35e`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/plan.md — SHA-256 `5da953eca4309b0d8aa4a33cdaa70b1ac8f930f51b9c856dbea1e0459f9810f5`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/tasks.md — SHA-256 `4e2d88149deaad7e25211d94cf364c9dffae88ff396a2380b50077e0f76ff212`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/reuse-audit.md — SHA-256 `4d2d39aa9dc1ec13b924158ed4b7a5b46a8d8ab987da0560ba41706527ebf47b`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/contracts/presentation.md — SHA-256 `b009057c06d133130cf7b0b555923473572aedcbcc1d9ba7b2a7022428c79e7b`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/tests164.md — SHA-256 `bf68269ed5809cefb44fc135c868ff272e7f81863ed8217171cf48263a46fdb0`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/plan-review164-r1.md — SHA-256 `8c20bb1c7c923fa7d9acd06fe038acf2c318e935d8972763aa9b26620f7edd50`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/requirements-matrix.md — SHA-256 `78c27814b6c0845d97ae214d03b4f9ae92ed3392a230191e7e67f6ce48788878`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/checklists/requirements.md — SHA-256 `80eb89ec35a61ec9be09f480d4b4f81d7d7ddbbf5f8a753ca6b73f8bb3a2066e`.

<a id="extrait-4"></a>

## Extrait 4

Le réemploi, les limites4096/64/100/16KiB et la frontière T3/Bridget restent cohérents. Le tri unique est explicitement O((n+m)log(n+m)), distinct de la projection O(n+c). Le fichier vite.config.ts est une entrée ciblée de liste interop, règle globale inchangée. Aucun critère de recette ou livraison n’est levé.

<a id="extrait-5"></a>

## Extrait 5

Le rapport `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p1.md` contient chaque réalisation fichier/ligne, preuve et limite. Prochaine lecture après T021 ; contre-revue indépendante ensuite.

<a id="extrait-6"></a>

## Extrait 6

Rapport et lignes source : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p2.md`. Contre-revue/audit T014 et recette/livraison T015–T020 restent ouverts. In Progress2/21, aucun cochage.

<a id="extrait-7"></a>

## Extrait 7

Aucun finding documentaire supplémentaire actif. La correction sourceT022 reste ouverte jusqu’à preuve réelle. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r1.md`.

<a id="extrait-8"></a>

## Extrait 8

Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md`. SourceP3CONVERGED ; contre-revueR2/auditdelta et gatesproduit futures. InProgress2/22, aucune case cochée.

<a id="extrait-9"></a>

## Extrait 9

**P3 CONVERGED sur la source ; R2 APPROVE.** F1/F2/F3 sont fermées. Le compteur représente des occurrences, pas des versions uniques. F4 est accepté et non relu. Rapport et limites : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`.

<a id="extrait-10"></a>

## Extrait 10

Audit incrémental de six sources,309lignes lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01`. GradeA100 ciblé,0finding actif ; validateur canonique RC0,0erreur/0warning. AuditR5b, P1/P2 et tasks conservés byte-identiques. Aucun nouveau test ni contrôle source relancé. Les28PASS et trois TypeScript RC0 proviennent du manifeste finalR6, pas de la lecture documentaire.

<a id="extrait-11"></a>

## Extrait 11

Le build actif Bridget est toujours source `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` ; le commit de métadonnéesc57 est distinct et ne prétend pas une nouvelle compilation. Source T3 `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée au fork/local/main-20261009, application164 signée sur disque. T3PID704/969 reste chargé en162, sans redémarrage ni activation différée.
