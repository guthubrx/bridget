# Correction 263

Version 0.8.29, identité 263-coordination-resultats-r2. Corrections portables, sans chemin personnel ni nom de machine imposé dans le produit.

## Causes vérifiées

- Guidage de l’assembleur : un résultat avait été remis quatre secondes après le dernier steer accepté. Les deux refus suivants concernaient donc une ronde déjà finie. Le nouveau refus indique la tâche existante, sa lecture et la continuation native éventuelle. Il ne transforme pas le motif hors projet en autorisation.
- Vue locale : le registre du worker attesté était sélectionné pour shared hors Git seulement. La préparation locale commit_view pouvait consulter un autre profil. Le même registre attesté est maintenant utilisé pour cette vue. Identités, empreintes et mappings restent vérifiés ; aucun registre n’est réinitialisé.
- Web : les définitions réellement lancées pour les deux missions discovery interdisaient WebSearch, WebFetch et Bash. La machine n’était pas la cause. Le contrat MCP et le catalogue expliquent cette limite avant admission. Le catalogue ne certifie pas les outils web, les données ou les hôtes SSH externes. Ces ressources ne sont pas transférées avec le dépôt ; une contrainte de destination doit être explicite quand elle est nécessaire.
- Résultat incomplet : le pilote effaçait les réponses achevées en attendant les tâches de fond. Il conserve désormais ces textes et les réunit une seule fois, en O(total des textes), avant le résultat durable. L’attente des tâches de fond et l’échéance initiale restent applicables.
- Journal après arrêt : la façade suivait un flux actif et rendait seulement AgentStopped. Elle explique maintenant la portée de cette lecture et l’accès à l’historique conservé dans la vue native, sans proposer de relancer une mission pour le lire.
- Limite de préparation : trois contrôles perdaient leur cause connue. Ils rendent désormais operation_capacity ou metadata_capacity sous le contrat déjà négocié. Le reçu ancien ne conservait pas les compteurs permettant de départager ces contrôles. La tâche annulée n’est pas rejouée pour reconstituer cette observation.

## Faits non établis

Les deux envois suspectés conservent le destinataire attendu et des corps non vides. Le journal de l’autre enfant contient une seule entrée reçue, non vide, de 5083 octets. Aucun défaut de routage ni perte du corps n’est démontré. Les preuves techniques privées conservent les identifiants et empreintes ; aucun texte métier n’est publié.

La coupure de transport signalée précède la maintenance de 262. Les traces conservées ne permettent pas d’en établir le type d’erreur système. Le client journalise maintenant l’étape connect/write/read, le type fermé de l’erreur et la date. Aucun chemin, compte, secret ou corps n’est ajouté à ce diagnostic. La reprise conserve les identifiants et les droits.

## Revue et source exécutable

Revue indépendante en lecture seule : tâche 87bb84ef-ff37-472e-9318-45e4d1f288dc, clé bridget-263-result-message-review-20261111-r1. Le défaut de conservation est confirmé. Le signalement de message vide n’est pas confirmé. La mission de revue a rendu son résultat ; l’annulation propriétaire est enregistrée et l’arrêt de son enfant est confirmé. Sa vue reste conservée selon le contrôle natif.

La décision D1 du programme 260 reste respectée. build-runtime.py reconstruit la base active 75259217a72b3177cc0f50ba58c1fc3f97e5e22e avec runtime.patch. La source obtenue reproduit exactement les 27 fichiers attendus. Les extractions réservées à une activation ultérieure restent hors du binaire.

## Validation et activation

Les sept services sont chargés avec 263-coordination-resultats-r2 sur quatre hôtes. La coupure finale du maître a duré 2,29 secondes. Le lien natif est revenu et le catalogue actif expose le guidage livré. Les exécutables réellement chargés et les pointeurs de retour ont été contrôlés. Le retour 262 reste qualifié pour les formats conservés. Les anciens superviseurs des missions encore en cours sont préservés ; cette livraison ne les remplace pas et ne rejoue pas leur travail.

Le parcours réel de la vue locale Mac est qualifié : la nouvelle revue 4652264b-beea-4e68-94d7-c8eaf01208eb, clé bridget-263-portable-mac-review-20261111-r2, est admise et rend son résultat. Aucun défaut prouvé dans la garde ABI. Les trois tests ABI sont aussi raccordés au workflow d’hygiène du dépôt.

Les 81 scénarios Rust ciblés par cible Mac et Debian 12, ainsi que les 11 tests Python, passent. Les tests voisins encore valides sont réutilisés. L’analyse statique stricte réussit sur les deux plateformes. Le paquet Android 8029 garde sa lignée de signature. Aucun élément graphique ne change dans ce lot.

La release 0.8.29 est publiée, son tag pointe e2d65d253ba55f3665e1518573244218b735116e. Les six actifs ont été téléchargés sans authentification et comparés à leurs empreintes. L’image Linux commune, construite sur Debian 12/glibc 2.36, exige au plus glibc 2.34. Le premier candidat Linux plus récent a été refusé avant activation sur la cible commune et remplacé dans la distribution avant publication.

Les sept avertissements Linux préexistants de la base ont été corrigés sans changement de comportement : conversions entières explicites communes aux plateformes et conditions let regroupées. Ces ajustements ne déclenchent aucune extraction réservée au programme 260.

La qualification de distribution a refusé le premier binaire Linux avant activation sur Debian 12 : il exigeait glibc 2.39. Le candidat commun est reconstruit sur Debian 12. Le script check-linux-abi.py impose désormais glibc 2.36 au plus à make release sur Linux ; une absence de preuve n’est pas un succès. Trois tests et un refus réel du premier binaire qualifient cette garde.


## Défaut découvert sur le parcours Mac réel

La nouvelle préparation locale franchit désormais le contrôle d’identité. Une revue de qualification a ensuite échoué pendant l’audit de la vue. Tâche 859861e4-4d51-4f78-91cd-d2119dd49792, clé bridget-263-portable-mac-review-20261111-r1 ; annulation confirmée, source arrêtée et matière conservée, aucune mission métier rejouée.

La lecture brute du manifeste et des fichiers ne montre aucune divergence de contenu ni de mode. Le contrôle Rust de cette même vue passe sous la limite de descripteurs du terminal, puis échoue avec EMFILE sous la limite 256 de launchd. Le répertoire comporte une fratrie de 260 dossiers. L’ancien parcours ouvrait tous les frères avant de les traiter.

Le parcours conserve maintenant un cadre par niveau : descripteurs O(profondeur), temps O(entrées + octets), noms lus une fois. Les contrôles par descripteur, les liens, la carte, les empreintes et le dépôt source restent vérifiés. La limite système n’est pas augmentée. Une régression isolée sous 64 descripteurs exporte puis audite 96 dossiers frères et confirme que le dépôt source est inchangé.

Revue indépendante du parcours borné : tâche 22436410-ab54-4188-a6c4-b8abae8c919a, clé bridget-263-fd-audit-review-20261111-r1. Aucun défaut prouvé. Les 45 tests de vue passent sur Mac et Debian 12, dont les exports volumineux et le cas sous 64 descripteurs ; trois helpers réservés aux sous-processus sont ignorés par le lancement ordinaire. Analyse statique stricte réussie sur les deux plateformes.


## Ménage gardé

Six anciennes copies exécutables sont retirées sur Mac, SYS et KS, soit 254497392 octets logiques. Les exports et copies de transfert retirés sur ces trois hôtes représentent 1172698802 octets logiques. Total du lot : 1427196194 octets, environ 1,43 Go. La distribution publiée, les reçus, les preuves natives et les travaux en cours sont conservés.

Le ménage reste partiel sur les périmètres suivants :

- Mac : entretien global des caches différé, car le cwd du processus 33346 ne peut pas être observé. Aucun processus d’un autre travail n’est arrêté pour nettoyer.
- Cartae : trois copies anciennes et les exports propres restent conservés, car l’observation complète des usages a refusé des descripteurs transitoires ou non qualifiables. Le dernier contrôle des copies désigne le processus 3326905, descripteur 52 ; celui des exports désigne le worker 1843131, descripteur 7. Un descripteur du contrôle précédent avait disparu lors de la lecture suivante. Ces constats ne prouvent pas que toutes ces copies sont utilisées. La garde n’est pas affaiblie et aucun retrait n’est forcé.
- SYS : une ancienne copie 246-reader-r1 reste conservée, car son dossier n’est pas privé et aucune observation globale privilégiée n’est disponible pour son retrait.
- Les images encore chargées par des missions actives restent conservées avec leurs travaux et reçus. Les nouveaux lancements utilisent le binaire corrigé.

Aucune branche ni aucun worktree n’est créé par ce lot. Les sources de livraison sont poussées et les trois checkouts distants sont alignés sur le commit exécutable. Le checkout Mac contient en parallèle les changements non intégrés du lot 264, propriété d’un autre responsable. Ils ne sont ni nettoyés ni inclus dans le binaire 263.


## Rechargement des clients et retours

T3 a été fermé normalement puis rouvert, sans remplacement de l’application ni arrêt forcé. Le serveur passe du PID 16727 au PID 47529 ; la version de l’application reste 0.1.7-local.20261111.bridget-toolbar. Les cinq façades MCP observées chargent l’image 263-coordination-resultats-r2. Le lien natif répond avec ce même build. Les reprises automatiques ont conservé les conversations et moteurs existants ; quatre messages de reprise devenus superflus ont été retirés de la file, sans toucher aux autres messages.

Retours détaillés envoyés aux deux auteurs des signalements après le rechargement : messages mcp-54924-6af44940-1 et mcp-54924-6af44943-2, livraisons 7c5f635e-e2c7-460d-a812-de870bd13fb5 et 20687a0b-bbfb-4953-be57-f6979f11846e. Les reçus attestent une remise en vol, pas leur lecture. Les enveloppes exactes sont conservées ; aucun renvoi supplémentaire ni mission de remplacement n’est créé pour forcer un accusé.
