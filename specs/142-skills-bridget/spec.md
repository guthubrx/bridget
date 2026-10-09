# SPEC142 — Compétences Bridget lisibles et compatibles

Date : 2026-10-07. Statut : Implemented — publié, non committé. Session autorisée par l'utilisateur.
Tests : 16/16 synchronisation PASS par l'équipier puis rejoués par le principal sur la source publiée à 08:46 CEST ; 152/152 boucle isolée PASS par l'équipier. Les sondes de découverte ne sont pas des tests de fonctionnement du modèle.

## Objectif et périmètre

Présenter trois compétences sous les noms `bridget`, `bridget-loop` et `bridget-handoff`, dans Codex et Claude. Décrire leur rôle en français. Garder les trois anciens noms utilisables. Corriger les appels documentaires qui utilisent encore ces noms. Publier les instructions depuis une source choisie, sans réintroduire un ancien protocole.

Aucun changement de moteur, transport, daemon, T3, missions, modèle, API ou politique de boucle. Aucun redémarrage. Aucun lancement de modèle payant. Aucun commit, fusion ou push n'est autorisé par cette seule préparation.

## Histoires utilisateur

### US142-01 — Reconnaître les compétences

En tant qu'utilisateur, je veux lire des noms et descriptions français cohérents afin de choisir la bonne compétence dans Codex et Claude.

- Étant donné le catalogue Codex, quand les nouvelles compétences sont découvertes, alors leurs identités sont les trois noms canoniques et leurs métadonnées sont françaises.
- Étant donné Claude, quand les instructions sont publiées, alors les mêmes noms canoniques et descriptions de rôle sont présents dans les fichiers découverts.
- Les trois alias restent des entrées distinctes. Le catalogue n'est pas promis limité à trois lignes.

### US142-02 — Conserver les anciens appels

En tant qu'utilisateur, je veux conserver mes anciens appels afin que le renommage n'interrompe pas mes habitudes ni une boucle active.

- `$agent-bridge`, `$agent-loop` et `$agent-handoff-ledger` restent des alias explicites vers les nouvelles compétences.
- Chaque alias garde son identité historique et indique en français la compatibilité.
- Les anciens scripts gardent leurs chemins et leur contenu. Le LaunchAgent actif garde sa configuration.
- L'alias `agent-bridge` ne réactive jamais `bridge.sh` ni l'ancien protocole AgentBridge.

### US142-03 — Éviter une régression lors de la synchronisation

En tant que mainteneur, je veux une source explicite afin qu'une copie ancienne plus récente ne remplace pas les instructions validées.

- Le publisher traite les six identités par une branche dédiée sans promotion fondée sur la date.
- Une seconde publication ne change pas le contenu ni les scripts conservés.
- Les sauvegardes historiques sortent des racines de découverte sans suppression de leur contenu.

### US142-04 — Corriger les références et vérifier la livraison

En tant qu'utilisateur, je veux que les autres compétences utilisent les nouveaux noms sans casser leurs commandes réelles.

- Les huit fichiers appelants ciblés emploient les noms canoniques dans leurs instructions.
- Les chemins CLI historiques de `horizon-new-episode` restent inchangés.
- La publication utilise une sauvegarde vérifiée et rapporte les chemins exacts. Elle distingue découverte, publication et exécution réelle.

## Exigences fonctionnelles

- FR142-01 : publier les identités canoniques `bridget`, `bridget-loop`, `bridget-handoff` pour Codex et Claude.
- FR142-02 : fournir des descriptions françaises qui distinguent communication, boucle bornée et transmission de contexte.
- FR142-03 : renseigner `agents/openai.yaml` avec un nom lisible, une courte description de 25 à 64 caractères et un `default_prompt` contenant le nom canonique avec `$`.
- FR142-04 : conserver le comportement de la compétence Bridget du dépôt Bridget. Renommer la présentation des autres compétences sans modifier leurs règles opérationnelles.
- FR142-05 : garder les alias `agent-bridge`, `agent-loop`, `agent-handoff-ledger` distincts, visibles, invocables et clairement marqués compatibles. Ne pas ajouter `user-invocable: false`.
- FR142-06 : donner aux alias un pointeur absolu vers le nouveau canon. Interdire explicitement l'ancien transport AgentBridge dans l'alias `agent-bridge`.
- FR142-07 : préserver les scripts historiques, leurs permissions, chemins et cibles. Dans les nouveaux dossiers loop/handoff, utiliser `scripts -> ../agent-loop/scripts` et `scripts -> ../agent-handoff-ledger/scripts`.
- FR142-08 : préserver le LaunchAgent actif et son appel `/Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py`.
- FR142-09 : choisir une source unique explicite par compétence. Synchroniser les instructions vers Codex et Claude sans importer les copies obsolètes par date.
- FR142-10 : étendre le publisher existant pour les six identités. Préserver les comportements des autres compétences et refuser les sources manquantes ou cibles dangereuses avant écriture.
- FR142-11 : sortir les dossiers `agent-bridge.pre-104-*` des racines de découverte vers une sauvegarde hors catalogue, sans perte de contenu.
- FR142-12 : corriger les huit fichiers appelants ciblés. Préserver les deux commandes CLI historiques de `horizon-new-episode` dans chaque fournisseur.
- FR142-13 : vérifier la publication par tests isolés : absence de promotion obsolète, non-destruction, sauvegarde, liens, parité des instructions et idempotence.
- FR142-14 : sauvegarder les cibles actives avant publication réelle. Vérifier ensuite contenu, alias, liens et scripts. Ne pas redémarrer de service ni lancer de mission.
- FR142-15 : consigner la preuve native de découverte disponible et ses limites. Ne pas déduire une exécution du modèle d'une réponse `skills/list`.

## Critères de succès

- SC142-01 : les trois nouvelles identités existent pour les deux fournisseurs et les six identités ont des descriptions de rôle françaises.
- SC142-02 : les trois anciens noms restent utilisables comme alias explicites ; aucune désactivation d'invocation n'est ajoutée.
- SC142-03 : les chemins et empreintes des scripts actifs, ainsi que la configuration LaunchAgent, restent identiques avant/après publication.
- SC142-04 : les tests isolés du publisher passent, y compris une copie legacy rendue artificiellement plus récente et deux publications successives.
- SC142-05 : les huit fichiers appelants sont corrigés, les quatre lignes de commandes CLI conservées, et aucun dossier de sauvegarde pre-104 n'est découvert comme compétence active après publication.
- SC142-06 : les instructions publiées correspondent à leur source explicite ; un compte rendu donne sauvegarde, chemins, contrôles et limites sans revendiquer une exécution de modèle non faite.

## Hypothèses et cas limites

Codex utilise `openai.yaml` pour humaniser l'affichage. Claude peut ignorer ce nom d'affichage ; les noms legacy et descriptions de compatibilité doivent donc suffire. Une source absente bloque la publication ciblée. Un lien ancien peut viser un runtime externe : il ne doit pas être remplacé sans conserver sa cible et ses scripts. Deux dossiers au même `name` ne sont pas une solution de compatibilité. Une simple seconde liaison symbolique vers le canon peut supprimer l'identité historique dans le catalogue.

## Préparation et estimation

Sync projet appliqué par le principal. Les scripts et templates SpecKit locaux sont absents ; les artefacts sont produits depuis les règles utilisateur lues. Sauvegarde de préparation : `/Users/moi/.cache/bridget-skills-142.eTqmgl/`. Estimation globale indicative : 25–40 minutes, selon les contrôles de publication. Une preuve nouvelle actualisera le journal, pas une hypothèse.
