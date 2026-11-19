# Quota du tour Codex et placement du compte Pro

Incident Parix mcp-66364-6aff4160-31, le 19 novembre 2026.
Le maître conserve cinq tâches failed au motif terminal Codex: failed. Les
journaux fournisseur sur SYS conservent usage_limit_exceeded à 18 h 30 UTC
pour deux sessions, dont celle du dossier ec934da6. Le schéma app-server du
programme installé expose usageLimitExceeded dans TurnCompletedNotification.
Aucun texte libre du fournisseur ne sert à classifier le terminal.

Hypothèse confirmée par un test rouge. Le transport écarte la cause car sa
projection ne connaît que serverOverloaded et model_not_found. Le transport
corrigé rend usage_limit. Le statut conserve une date si les mesures de quota
de cette exécution l'attestent. Le registre maître de la tâche ec934da6 ne
contient pas d'opt-in. Le retour observé du compte est le 25 novembre à 22 h
CET, au-delà des six heures prévues même avec opt-in. Le contrat de reprise
existante, sa preuve de quota par instance et ses deux rondes restent la
référence. La délégation automatique n'expose pas ce paramètre.

Repli chef-parix-integrateur10-reprise-feux-relies-20261119-1936. Le maître a
une observation de refus à 18 h 34 min 40 s UTC et aucun reçu d'admission
persisté pour cette clé. Le profil codex-pro-models62 est déclaré, mais absent
d'allowed_agent_types pour monordinateur dans le registre maître et celui du
worker. Le profil codex-pro est autorisé. Le catalogue du Mac relu pendant
l'investigation projette can_run true pour les deux profils locaux. La
connexion Pro reporte gpt-5.6-luna, gpt-5.6-terra et gpt-6-luna. L'accès aux
modèles 6.2 sur cette connexion reste non vérifié. Ne pas attribuer le refus
de placement au quota ou au contrôle des permissions du tour parent.

Le correctif rend machine_agent_type_not_allowed si tous les candidats sont
exclus de la liste de profils de leur machine. Conserver la clé et les
contraintes du demandeur. Aucun registre actif ni historique de tâche modifié.
Aucun essai du modèle ni ouverture de profil en production.

Revue native en lecture seule, demande bridget-303-quota-placement-revue-20261119-r1,
tâche 9ff778d5-aebe-45c1-b555-8d9e1de5abfb. Sources du checkout principal, sans worktree.

La revue a relevé une dépendance inutile à la clôture d'exécution dans la
première version du correctif. Le chemin sans opt-in persiste maintenant
l'échec sans attendre ce rapprochement. L'appelant NativeFailed retire le
rappel de cette demande et laisse la notification durable inclure la date
lorsqu'elle est connue. Les observations d'une autre instance ne sont pas
utilisées. L'identifiant de session de la trace ec934da6 est 01a1f508-31bd-7842-a3c8-009a899e8fb1.
La revue a été lue via le port natif.

Les trois tests 303 passent après traitement de la revue. Les 55 tests
native160 passent. La suite transport Codex compte 84 succès et un échec
workspace_repair_busy dans un test de réparation de dossier. Ce test passe
seul, avec le même transport corrigé. Le premier essai du test maître
rapportait null à cause d'une assertion sur state au lieu de status. Le
test corrigé vérifie le statut, l'absence d'attente et les trois cas de date.

Activation contrôlée de nouveau sur Cartae. Le port updates prepare refuse
update_missions_active_or_unconfirmed. Aucun bail acquis. Aucun arrêt de
service ni relance des missions concernées. La cause historique a été lue
dans les journaux sans changer les enregistrements des tâches.

Clippy strict des bibliothèques et tests passe. Diff relu et sans erreur
d'espace. Cache final 8,22 Gio, sous le budget du dépôt. Aucun worktree créé.
