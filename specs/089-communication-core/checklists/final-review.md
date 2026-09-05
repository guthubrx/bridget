# T036 — Revue finale de la communication extraite

Date : 2026-09-05. Référence de réalisation : `42029fcc`.
Revue autorisée par l'utilisateur, indépendante de l'implémentation, en lecture
seule. Deux périmètres distincts : corpus/extraction ; sécurité IPC/namespace/SSH.
Les contre-lectures n'ont lancé ni fournisseur ni daemon de production.

## Verdicts et corrections

| Point examiné | Verdict et preuve |
|---|---|
| Autonomie des trois crates et ressources des tests | PASS structurel. Pas de source Maicie/UI/runtime manquante identifiée ; 264 dispositions uniques, sans test perdu démontré. |
| Corpus épinglé | PASS. Le relecteur a exécuté le vérificateur : 17 fixtures identiques aux objets Git. Le parent a rejoué les six mutants. |
| Annulation face à un destinataire bloqué | AMENDER initial, puis PASS ciblé du correctif. Transaction durable et retrait pending avant libération du verrou global ; notification ensuite, attente writer/écriture sous une échéance commune. Trois oracles de pression/bytes nominaux. |
| Sources du paquet via ancêtre symbolique | AMENDER initial, puis PASS. Contrôle de tous les parents avant création ; reproduction statique du contournement, refus sans copie ni mutation après correction. |
| Profil durable créé par chaque statut | AMENDER initial, puis PASS. Sonde CLI sans instance éphémère ; vrai daemon, trois CLI, compteurs des deux tables inchangés. |
| Flotte historique et SSH | Aucun contact implicite ni affaiblissement de clé d'hôte identifié dans les chemins inspectés. Tests de scripts et recettes distantes séparés dans implementation.md. |

Les résultats de tests rouge/vert sont ceux exécutés dans le chantier ; les
relectures des correctifs confirment le code et les oracles, sans prétendre
avoir exécuté une seconde fois toutes les recettes.

## Limites qui restent visibles

- Un ACK atteste une remise, pas la réussite du travail de l'agent.
- L'unicité après crash reste bornée au domaine daemon/wrapper documenté et à
  l'horizon d'idempotence ; pas de garantie universelle exactement-une-fois.
- Le délai d'annulation corrigé borne sa notification après commit. La revue
  ne prouve pas une borne de toutes les anciennes écritures serveur, ni du
  temps d'acquisition initial du verrou global ou de SQLite.
- Même UID et compte SSH autorisé ne sont pas des frontières d'isolation
  contre un opérateur local malveillant. Aucun retrait de permission de session
  n'est légitimé par la suppression du runtime de projet.
- T020 reste partielle sur l'authentification Claude ; T021 ne dispose pas
  encore de la preuve réelle GLM via Claude Code sur forfait. Ces absences
  ne sont ni des succès simulés ni une autorisation de repli vers une API payante.
- Les mesures de charge et SSH ont leur méthode et leur date ; elles ne sont
  pas des chiffres inventés pour la dernière compilation de documentation.

## Adoption

Le paquet et son guide permettent une installation privée côte à côte. La
validation complète du paquet et l'essai d'installation sont consignés dans
implementation.md. La bascule de la flotte, l'import des bases utilisateur et
la modification des profils/skills globaux demandent toujours une décision
distincte. Aucun de ces actes n'est réalisé par cette revue.
