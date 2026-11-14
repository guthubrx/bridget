# Libellé du délai du transport partagé

Signalement Devantures du 14 novembre 2026 : tâche f0fbb334-a173-4f0e-830b-2d09f057018e, ronde 2, moteur déclaré glm et modèle glm-5.3. Le maître confirme une remise à 15:16:49 Paris et une fin logique à 15:26:49, exactement les 600 secondes demandées. L’absence de résultat ne prouve pas un problème de quota ni une bascule de moteur.

Cause du mauvais nom : le transport claude_stream_json est partagé et écrit en dur « échéance Claude dépassée ». Le nouveau texte est « échéance du tour dépassée ». Le daemon reconnaît le nouveau texte et conserve les deux anciennes formes mission_reply_timeout et échéance Claude dépassée, pour garder le guidage des tâches déjà enregistrées. Aucun délai, état de tâche, droit, calcul ou lancement n’est modifié.

Qualification : test du transport réel simulé et projection du statut pour les trois formes, sur la copie cumulative du runtime. Les deux tests passent sur Mac. Clippy tous targets des deux crates avec -D warnings passe. Preuves : /Users/moi/.cache/bridget293-echeance. Aucune revue métier relancée, aucun Continue ni cancel exécuté pour cet incident.

Activation non effectuée : la maintenance native reste bloquée par les tâches et admissions inachevées constatées lors du lot 292. Aucun arrêt de T3 ni forçage de ces états. Le correctif reste absent du binaire actif tant que cette fenêtre ne peut pas être obtenue. Les anciens reçus sont conservés tels quels.

Le cache de compilation commun termine à 7,96 Gio, sous le budget de 10 Gio. La copie cumulative et les preuves sont conservées pour la future activation ; aucun nouveau worktree ni binaire installé. Aucun test Linux ou chargement réel de ce texte dans un service actif n’est annoncé.
