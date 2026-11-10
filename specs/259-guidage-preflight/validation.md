# Validation 259 — services activés et version publiée

## Bilan final du 10 novembre 2026

Source exécutable intégrée et poussée : 406466cb82f63691b9f116765fecd156530af645. Bridget 0.8.26 / 259-preflight-guidance-r1 est chargé dans les sept services des quatre hôtes. Les façades MCP déjà ouvertes gardent leur ancien binaire jusqu'au prochain chargement. T3 n'a pas été arrêté ; aucun rechargement global des clients n'est affirmé.

Le contrôle ciblé de 13:35:33 CET trouve toujours la tâche Roadie 890203d2-7fbc-4ec6-8ff0-3cb070ccdab5 en ronde 1/result_available, acknowledged_at=null et delivery_acked=0. Il filtre la tâche et son propriétaire, sans lire le résultat métier. Aucun ACK ou mission rejoué. Le défaut confirmé est le guidage absent pendant le contrôle préalable ; la cause précise du délai initial reste inconnue.

Les références des groupes sont validées et bornées : 16 identifiants au plus, 128 octets chacun. Les listes invalides, répétées ou trop grandes ne sont pas recopiées. FR02 précise qu'aucun nouvel essai n'est ajouté. Les récupérations de lecture préexistantes restent inchangées. La confrontation finale FR01–04 ne trouve pas de manque dans le code et les services, sous la limite explicite des façades ouvertes.

| Contrôle final | Mac | Linux SYS |
|---|---|---|
| Contrat 259, dont délai Unix réel de 10 s | 4 PASS | 4 PASS |
| Suite MCP | 135 PASS, 1 ignoré | 135 PASS, 1 ignoré |
| Régression ACK | 11 PASS | 11 PASS |
| Contrat après transmission | 7 PASS | 7 PASS |
| Suivi 230 | PASS | PASS |
| Construction release | PASS, 151,22 s | PASS, 2 min 21 s |

Les groupes se recouvrent ; ne pas additionner leurs nombres. Le test ignoré exige un script et un binaire AgentLoop externes explicites. Les 326 fichiers de crates et Cargo comparés sont identiques entre les sources compilées. Aucun fournisseur métier n'a été appelé. La revue APPROVE_WITH_CHANGES est lue et acquittée ; les deux objections retenues sont corrigées et testées. Le delta final est relu localement, sans prétendre à un second verdict indépendant. Aucun audit V15 nouveau n'est revendiqué.

| Image active | SHA-256 |
|---|---|
| Mac | ddd3935dab73c4366f4d4c71896846d6934da97561844e3a4416717310cdced0 |
| SYS, KS, Cartae | cc48b666c8c35e907065181120dee72e1e3b5aa000c4bab0974c6372fb449cde |

Le retour 258-native-composer-r1 est compatible dans les contrôles de formats aller, retour et inverse. Les quatre pointeurs rollback ont été écrits sous le verrou natif, puis les outils de mise à jour redémarrés. Les processus chargés et leurs empreintes sont vérifiés. Coupure du maître : 1,61 s. Les quatre rapports sont frais et up_to_date en 0.8.26. L'annuaire natif répond après activation.

Le registre Mac, les sept payloads protégés et les six processus T3 observés avec leur date de naissance sont conservés. Les wrappers utilisés restent en place. Les processus T3 actuels sont nés après l'installation 258 ; cette observation n'attribue pas leur relance. Les façades MCP 258 déjà ouvertes, dont celle du signalement Roadie, ne sont pas déclarées chargées en 259.

Publication : https://github.com/guthubrx/bridget/releases/tag/v0.8.26. Les six fichiers ont été téléchargés sans authentification et comparés par empreinte. Android : version 0.8.26, code 8026, signature et continuité avec 8025 vérifiées. Le delta Rust final ne change pas son arbre : le build qualifié est réutilisé. Aucun fonctionnement sur appareil n'est revendiqué.

Le Python Homebrew 3.15 ne pouvait pas importer pyexpat. La première installation s'est arrêtée avant tout changement de lien ou d'image. Les scripts ont ensuite utilisé le Python Anaconda 3.12 existant. Aucun réglage global de Python, de PATH ou de droits n'a changé. Les outils de mise à jour natifs fonctionnent avec leur propre Python système.

Le worktree Mac 259, sa branche et sa référence distante ont été retirés après vérification de l'intégration, de la clôture de la revue, de l'absence de fichiers ignorés et d'une observation complète des usages. Trois copies de transfert Linux et l'image Mac 257 inutilisée ont été retirées : 171 932 488 octets logiques. Actif 259 et retour 258 conservés. La commande installed a purgé les intermédiaires du staging Mac : son relevé final donne 108 971 725 octets, soit 0,10 Gio. Le cache SYS commun reste à 5,54 Gio. Les deux restent sous le budget de 10 Gio.

Le worktree de compilation SYS 259 est conservé : cinq processus du compte ne sont pas observables. Le worktree Android Cartae 259 est conservé : 61 processus ne sont pas observables. Aucun usage direct trouvé ne suffit pas à prouver leur absence. La vue de revue exige encore le contrôle natif de rétention. Les images historiques encore utilisées ou sans observation suffisante, preuves, archives publiques et travaux étrangers restent conservés. Aucun processus étranger arrêté pour nettoyer. La session distincte 260 reste intacte.

Les avis utiles avant et après maintenance sont déposés silencieusement auprès des propriétaires Roadie et Parix. Les reçus prouvent stored_without_wakeup, pas une lecture métier. Aucun accusé conversationnel ni réveil de modèle.

Preuves techniques et reçus : /Users/moi/.cache/bridget259-guidage-preflight. Repères : protection-proof.json, release-public-verification.json, roadie-ack-state-final.json, maintenance-after.json. Aucun composant visuel changé : diaporama de correction visible non applicable.

## Notes de qualification antérieures au bilan final

Les paragraphes suivants conservent les observations intermédiaires. Le bilan ci-dessus fixe l'état final.

ACK Roadie : la lecture technique ciblée du 10 novembre à12:42:56 CET trouve la tâche signalée sous son propriétaire, ronde1/result_available et acknowledged_at=null. Le résultat reste conservé. Aucun ACK ni mission rejoué. Le message complet lu dans l'activité source ne contient que le délai JSONL ; les deux échanges du plafond MCP étaient les seules voies préalables sans guidage. Le défaut corrigé est ce guidage absent. La cause précise du retard n'est pas établie.

Mac : test Unix de délai réel10s et coupuresPASS (2tests). Le serveur n'observe que CallerScope, puis EOF, sans ACK ni nouvelle connexion. Suite MCP élargie :135PASS, zéroéchec, un test AgentLoop ignoré avec son prérequis explicite (script+binaire local, jamais production). RégressionACK et construction release en cours au dernier relevé.

Deux échecs initiaux de qualification, déjà présents dans la source065f54f0, ont été corrigés : attendu de schéma160 incomplet pour le champ optionnel246 ; inventaire094 confondant les sous-commandes imbriquées avec les commandes racines, et entrée sentinel absente du tableau. Le parseur vise désormais les bras racines ; aucune commande autonome run/tick/activation ajoutée. La campagne244 reste arrêtée. Aucun champ ou commande de production ajouté par ces ajustements.

Linux : deux tests259, régressionACK et tests230PASS. Suite MCP élargie et construction finale en cours après raccord des mêmes ajustements de qualification. Source Mac/Linux comparée avant installation.

Réutilisation : helper pre_dispatch_failure, fixture registered196 et drop_scope196. Aucun nouveau délai, droit, helper, endpoint ou table. Les refus normaux et les issues inconnues après envoi restent dans leurs puits existants. Revue, Git, activation et nettoyage restent ouverts.

## Qualification finale avant intégration

La revue indépendante APPROVE_WITH_CHANGES est lue et son ACK acquis. Les objections retenues sont corrigées : tâchesdugroupe nommées avec rendu borné ; raccordementWait guidé sans modifier son attente3s. Le delta est vérifié localement, sans prétendre à un second verdict indépendant. Voir adversarial-review.md.

Macfinal :4tests259PASS,135MCPPASS/1AgentLoop ignoré avec prérequis,11ACKPASS,7tests post-dispatchPASS et contrôletracking230PASS. ConstructionreleasePASS151,22s aprèsdelta. Linuxfinal :4tests259PASS,135MCPPASS/1ignoré,7tests post-dispatchPASS,tracking230PASS ; construction distante terminée en2m21, visible danssonlog après coupureSSH du flux de compte rendu. La connexionSSH suivante a réussi ; ne pas assimiler cette coupure à un échec de compilation. Aucun fournisseur métier appelé.

Deux échecs du test privéLinux138 ont été vérifiés sur données réelles : socket créé2,488s aprèsPID, seuilhistoriquede1s. Seule la fixturecfg(test) utilise désormais le budgetexistant10s pour sa disponibilité ; la suite135passe après correction. Le premier essai de la nouvelle fixtureWait avait préparé sa credential avant de créer le socket, donc son helper restait sans effet : ordre corrigé,4/4tests259passent.

Confrontation finale des4FR au code/tests : conformité acquise pourFR01–03, publication et nettoyage encoreT004/T005. Aucun manque fonctionnel supplémentaire. Pas de nouvelle table, dépendance, endpoint, droit, durée ou politique de reprise.
