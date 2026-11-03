# Validation en cours

## Cause constatée

Les admissions `mails-015-ontology-review-20261103-01` et
`mails-016-references-review-20261103-01` ont échoué à `candidate_mapping`.
Le registre source contient les trois correspondances Linux du projet.
Le registre du maître ne contient aucune de ces correspondances. Le résultat
`ProjectPrepared` perdait le résultat structuré déjà vérifié par le helper.
Identité native et projet de communication présents. Les deux clés restent
absentes des tâches et des reçus d'admission au contrôle en lecture seule.
Aucune demande métier rejouée, aucun fichier métier changé.

## Correction

La capacité `origin_project_context_v1` négocie la transmission du contexte.
Sans cette capacité, requête et réponse conservent leur trame historique.
Le maître compare l'autorité cible observée avant la préparation avec le reçu
du worker d'origine authentifié. Il valide la forme du mapping, puis publie
sous le verrou natif et par l'écrivain atomique existant. Le writer relit
l'autorité et le registre avant publication. Aucun écrasement divergent.
Une racine additionnelle ne peut être importée sans approbation existante
sur cette cible dans le registre du maître. Les droits du parent restent
bornés par son cwd attesté, y compris dans un worktree lié.

Un ancien worker conserve les projets déjà connus du maître. Son succès
sans contexte ne prouve pas le raccordement d'un nouveau projet : ce cas
nomme la correction à charger, sans déclarer identité ou projet inconnus.
La liste des capacités reste fermée ; sa borne passe de 16 à 32 pour porter
17 capacités connues. Aucune condition sur un numéro de version fournisseur.

## Contrôles acquis

- Quatre nouveaux cas Python ciblés : préparation et publication sur deux
  registres distincts, reprise identique, worktree, autorité/racine/droit
  additionnel divergents et absence d'écrasement.
- Module de préparation complet : 47 cas PASS en 14,714 s.
  Premier passage Mac : deux fixtures de verrou refusaient le lien symbolique
  du dossier temporaire système. Reprise avec TMPDIR physique `/private/tmp` :
  47 PASS, aucune garde du produit affaiblie.
- Première chaîne Rust : reçu sérialisé, validateur natif, helper réel et
  résolution du mapping dans un registre maître initialement vide PASS.
  Une compilation initiale a signalé un import Path manquant, corrigé.

Assemblages, revue et activation restent à consigner. Preuves privées :
/Users/moi/.cache/bridget229-project-relay.

La première demande de revue de cette correction avait reply_timeout=1, erreur
de l'auteur de l'appel : ce champ est la durée de mission, minimum 60 secondes.
Refus durable `reply_timeout_out_of_range`, sans tâche, conservé sous sa clé
`bridget229-project-relay-review-20261103-01`. Aucun enfant lancé. Le message
générique était insuffisant : guidage précis ajouté et vérifié séparément.
La demande corrigée est distincte de ce refus ; aucune mission inconnue rejouée.
Les deux appels historiques 29.Mails ont aussi des durées 10 et 1 secondes.
Leur refus de mapping a précédé ce contrôle de durée. Ce paramètre devra être
corrigé par leur propriétaire avant une admission, selon leur état durable.

## Traitement de la revue

Revue native c2726ed4, Sonnet choisi automatiquement sur KS, résultat lu,
ACK et clôture propriétaire effectués. Pas de P1 ; deux P2 et trois P3.
Lecture statique partielle, sans diff Git disponible et sans tests exécutés
par le lecteur. Les limites de cette revue sont conservées.

- P2-2 retenu et corrigé : comparer l'autorisation source de la racine
  additionnelle, pas sa destination privée propre au projet. Essai positif
  avec deux destinations distinctes et essai de changement de mode refusé.
- P2-1 : restriction proposée aux seules clés auto et dossiers gérés non
  retenue. Les correspondances explicites préprovisionnées sont aussi valides.
  La source du reçu n'est pas un argument de mission : worker d'origine
  authentifié, helper natif, contrôle du dépôt commun et validation SSH de
  la destination, puis égalité au registre source relu. Une racine `/`
  substituée dans une préparation normale ne passe pas le contrôle du dépôt
  commun dans prepare(). Une réponse arbitraire nécessite de compromettre
  ce worker de confiance. Le maître garde le cwd attesté et les sondes de la
  cible avant toute mission. La validation lexicale des chemins distants
  dans le maître est volontaire ; il ne possède pas leur système de fichiers.
  Aucune nouvelle restriction de placement légitime n'est ajoutée.
- P3-1 traité par un essai natif supplémentaire : vraie connexion locale
  corrélée du worker authentifié, prepare_project du maître, writer réel,
  publication puis résolution ; six cas : succès, mauvaise autorité, clé
  imposée différente, mapping invalide et deux replis ancien worker.
  Aucun modèle, service ou tâche créé. Le désaccord de clé rend désormais
  project_mapping_mismatch, plutôt qu'un diagnostic d'identité machine.
- P3-2 : guidage des autorités divergentes précisé, dépôt préparé conservé.
- P3-3 : commentaire sur la négociation au moment de l'échange ajouté.

Dernier module Python : 48 PASS en 29,356 s. Quatre cas Rust229 PASS en
0,32 s, dont le parcours maître à six variantes. Deux anciens contrôles du
helper et le contrôle Git d'origine restent acquis, soit sept tests Rust
distincts utiles pour ce lot. Le dernier delta de guidage ne change pas
les validations. Réassemblage final nécessaire pour le helper embarqué.

## Passage des services

Source exécutable finale ffe7896b, version0.7.8/build229-project-relay-r3.
Assemblages Mac et Linux Debian12 acquis. Empreintes :
Mac 5d7eb366e5ed1183ca9aa30e78ac9f5124720a207b90db5e5c725ca72acf5cc1 ;
Linux 961352eb7e8bce2360bae0139834a3bf178cd6aa479dc195ac4b831bbfb64e73.
Sept services chargés et empreintes relues sur quatre hôtes. Coupure du maître
2,12 s. Superviseur Cartae2500061 conservé avec sa naissance système identique.
Registre Mac spécial inchangé. Retours228-r3 compatibles dans les deux sens,
pointeurs rollback écrits par le writer natif sous verrou sur les quatre hôtes.
Les updaters ont repris. Installed Mac et SYS conservent les caches lorsque
l'observation des processus est incomplète ; aucun nettoyage forcé.

La tâche7024 conserve intégralement son payload,
SHA24a44aa30255e0be97bd29ffea3c4ed2e77d6a0cfad8c71d4992a7adc6050887.
Aucun rejeu, annulation, changement de génération ou service métier Opus2D.
La revue T3 métier distincte reste hors de cette maintenance.

APK0.7.8/code7008 signé avec le certificat et la lignée de production existants,
SHA573176d52ebb69c472f6c7871ae35be3a3c7d10d242f4a8097b78081fab5ad66.
Signature v3 comme le paquet de référence ; aucune nouvelle recette UI ou FCM.
La préparation Android a conservé ses erreurs de configuration : JDK absent
sur le Mac, SDK non indiqué sur Cartae, paramètres de signature incomplets.
Construction finale sur Cartae avec les outils existants, sans
installation ni changement de certificat. Paquet et six actifs de release
vérifiés par taille et SHA avant publication.

Neuf façades MCP T3 sont encore228-r3 au contrôle avant reprise. Elles ont été
conservées jusque-là ; rechargement natif de T3 et vérification du lien encore
nécessaires avant de qualifier la livraison complète. Dix-sept agents concernés
prévenus une fois, aucun accusé demandé. Suite dans le reçu de reprise privé.

Nettoyage avant rechargement client : huit copies de binaires inutiles,
342037056 octets logiques, retirées sur les quatre hôtes après contrôle d'usage
ciblé sans fichier ouvert ni diagnostic incomplet. Candidat229-r2 jamais activé
et transferts229 supprimés. Actif229-r3 et retour228-r3 conservés. Branche229
intégrée, retirée localement et au distant. Caches récents et vues gardées :
observation globale des processus incomplète, retrait natif non qualifié.
