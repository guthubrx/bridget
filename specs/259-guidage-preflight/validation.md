# Validation 259 — en cours

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
