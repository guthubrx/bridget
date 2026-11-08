# 251 — Présentation du client et placement local Parix

## Demande et exclusions

François demande de réutiliser les composants réels de conversation du client,
de raccourcir les titres et de préserver la lecture seule. Il interdit l’arrêt
de T3 sans assentiment explicite. Aucune mission métier n’est rejouée.

## Présentation

Les sources du client sont intégrées à `main` et poussées sur son dépôt personnel,
commit `75c7dafe51f8f734b96efbe77dbbbcecc5633235`. Le rendu passe par le vrai
ThreadTimeline : outils repliables, durée attestée, Markdown, code coloré et
contrôles communs. La politique existante des titres est partagée, avec borne
de 72 caractères et texte source conservé.

504 tests ciblés et 6 contrôles du compilateur React passent. Lint, Knip et
génération GraphQL passent. Les vérifications TypeScript ciblées web et serveur
passent. La vérification étendue reste en échec sur Cursor, avec sortie composée
d’avertissements/suggestions après correction du fixture périmé ; aucun succès
général n’est affirmé. La revue statique du raccord natif a été lue et acquittée :
ordinal constant corrigé et testé ; hypothèse d’erreur absorbée par une activité
écartée par le contrat et le test d’erreur terminale.

Paquet local 0.1.7-local.20261108.bridget-pages préparé, archive ZIP vérifiée,
signature ad hoc vérifiée. L’application installée n’a pas été remplacée.
Le diaporama versionné dans le dépôt client montre les vrais composants sur
données de démonstration, avec quatre clics relevés. Il ne remplace pas le
contrôle d’un sous-agent dans l’application après activation autorisée.

## Placement Parix sur monordinateur

Incident du coordinateur Parix : `project_mapping_unknown`, clé
`parix-experiences-dialogues-diffusion-20261108-01`, aucune tâche admise signalée.
Cause lue : correspondance existante sur Cartae et SYS, absente du worker Mac
et du registre maître pour monordinateur.

Publication de la clé existante `auto-54c0347616de987c5493e28c7690a30a`, dans
les deux registres, sous le verrou natif avec sauvegarde et écriture atomique.
Source et destination : `/Users/moi/Nextcloud/10.Scripts/73.parix`.
Identité Git : `https://github.com/guthubrx/parix.git`. Révision 1.
Le dépôt commun du worktree demandé et le commit 489d6bb ont été vérifiés.
Le contrôle natif de préparation rend `needs_bundle=false` et la correspondance
est relue identique. Aucun dossier créé ou déplacé, aucun service redémarré,
aucune permission fournisseur changée, aucune admission métier lancée.

Registre worker Mac après publication :
`60acca93483698c25d5b1aca4b52af20cd3ce24889b6ecf21ade87e2367daebe`.
Registre maître après publication :
`660b3ddf62504225bbddbbb15875706e0c4a2b1dc93a1a4d1d8c6d2620cd37b6`.
Ne pas restaurer les anciens registres à l’aveugle. Les autres correspondances
sont conservées. Le coordinateur reçoit le fait réparé et conserve son rôle
de propriétaire de la demande. Aucun succès d’exécution future n’est déduit.

## Signalement Mails : portée de shared

La préparation distante transporte les objets Git, pas les fichiers non suivis,
ignorés ou non committés du Mac. Shared désigne le checkout de la machine choisie.
Un HEAD identique ne garantit donc pas la présence de deux fichiers locaux.
Le résultat de la revue ea57a4db reste non concluant. Le demandeur reçoit ce
diagnostic et les deux possibilités : même checkout Mac autorisé, ou fichiers
utiles publiés avant une nouvelle admission autorisée. Aucune mission remplacée,
aucun accès aux contenus métier. Le défaut de clarté du guidage reste identifié ;
aucun transport nouveau ni refus préalable nouveau n’est déclaré livré.

Les refus tardifs de communication parent/enfant signalés par Parix ne sont pas
rejoués. La relation de mission active n’autorise pas à déduire une autorité
après fin de mission. Aucun défaut distinct n’est prouvé par leur seul reçu.

## Reçus privés

Preuves, demandes, reçus de publication et diagnostics :
`/Users/moi/.cache/bridget251-conversation/`. Ils restent hors Git.
Le code et les services Bridget 0.8.16 ne sont pas modifiés par ce lot.
