# Mise à jour officielle Claude Code sur Cartae

Autorisation utilisateur : « Mettre à jour et qualifier Claude Code sur Cartae ».

- Avant : 2.1.251 ; après : 2.1.312, chemin /home/moi/.local/bin/claude.
- Installation native officielle ; comptes, modèles, configuration shell et services Opus2D inchangés.
- Le script distant a installé la nouvelle version puis signalé shell_profile_symlink_requires_manual_path_configuration. Aucun profil lié n’a été modifié. La commande officielle claude update confirme ensuite 2.1.312 à jour ; claude --version confirme la version installée.
- Ancienne version conservée pour retour : /home/moi/.local/share/claude/versions/2.1.251.

## Cause historique attestée

Les deux journaux corrélés de 68a542c7-7113-4c60-954a-e7ba8ed32e85 et 3308346f-4ac7-4c37-942e-1b1ce3d425c3 donnent API400 : Claude Code2.1.251 ne prend pas en charge ce modèle, version2.1.280 minimale requise. Stderr : unrecognized_model pour claude-opus-5-5. Aucun problème de quota, compte ou durée14400 n’est établi.

## Qualification réelle via Bridget207

- Requête : bridget208-qualification-opus-cartae-20261029-02.
- Tâche : 2991e808-b9a3-4753-96f9-314aad6fd56f.
- Machine Cartae, connexion claude-cartae, moteur claude, modèle claude-opus-5-5.
- Résultat réel lu : BRIDGET208_OPUS_CARTAE_OK, erreur nulle.
- Résultat acquitté et clôture propriétaire demandée nativement.
- Premier essai de qualification refusé sans tâche pour mauvaise contrainte de connexion claude ; connexion corrigée depuis le catalogue natif, sans substitution de modèle.
- Notification T3 au fil monuments32e1b8f8 : clientRequestId bridget208-claude-qualifie-t3-20261029-01, remise steered au run528. Les tentatives Bridget n’ayant pas abouti ne sont pas comptées comme remises.

Cette qualification valide le programme Claude mis à jour et le modèle via Bridget207. Elle ne prouve pas le chargement du correctif208 distinct. Les anciennes tâches failed ne repartent pas par leur ancienne clé ; leur clôture et leur dossier doivent être vérifiés avant une nouvelle admission.
