# Découverte provisoire 159B

**Source non figée.** L’inventaire capture 59 fichiers : 53 suivis modifiés et 6 nouveaux. Il contient 174 hunks au moment de la capture. Les noms et les hashes des59fichiers sont stables avant/après cette lecture, pas garantis après poursuite du travail ROOT.

Le socle source compte 293 fichiers Rust/Python/TypeScript après exclusion des audits générés ; ce compte sert au mode M. Il ne mesure pas le code lu. Aucun buildmarkerUI ou CI n’est détecté dans l’inventaire suivi. Les sources Codex/Claude attestent une intégration fournisseur, pas un modèle actif.

## Register mécanique

26 fichiers contiennent seulement 34 ajouts queue_fallback:false. Les hunks de ces fichiers ont été lus avec leur contexte. Ils restent dans le scope, sans crédit de lecture de leurs corps entiers ni de tests exécutés. Les autres occurrences sont mêlées au code métier et attendent les checklists. Le nombre28 proposé avant inventaire n’est pas repris comme fait.

## Hot paths

Les huit points à contrôler sont les gardes de QueueEdit, les trois mutations et leur reçu, le plafond MCP, le DDL ouvert sur v7 et le repli Codex sans interrupt implicite. Les signatures/prologues ou extraits indiqués dans manifest.json ont été lus. Les corps complets restent à auditer. Le handler Promote est encore provisoire ; son refus actuel ne devient pas un finding artificiel avant l’implémentation prévue.

## Threat Model

Actifs : identité de connexion, droits hérités, canon immuable, reçu de rejeu scoped, phases/rangs/réservation et attribution des avis. Attaquants considérés : client non attesté, autre agent ou enfant sans droit, fournisseur ambigu ou panne/rejeu. Surfaces : Unix QueueEditRequest, MCP/CLI, transaction Immediate, copie steer-or-queue.

Risques hérités : preuve humaine locale limitée selon ADR011 ; garanties de remise réduites après libération et Seen. Ces limites ne valent pas résultat d’un test de sécurité. Aucun nouveau contrôle humain ou avis universel n’est ajouté.

## Limites

Promote/SC014 et des métadonnées sont encore en construction. Aucun résultat de tests, scanner, fournisseur réel ou modèle ne vient de cette préparation. Aucun finding, score ou GREEN global produit. La baseline canonique est absente, sans suppression inventée. La lecture par hash seule donne zéro crédit de lecture source. Les futures lectures doivent porter SHA, dates et plages, avec distinction historique/fraîche.

Avant Phase1 thématique, ROOT doit donner le gel final ; rafraîchir le diff et les hashes, puis charger module10 et les checklists.
