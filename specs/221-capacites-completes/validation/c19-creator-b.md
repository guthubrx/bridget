# Créateur C19 — services activés, qualification native ouverte
Source de départ : d35452df8789d61eccce54447e97fcb6f04229ba.
Le lecteur A était actif pendant les contrôles source ci-dessous. Le créateur
B a ensuite été activé ; les preuves et les limites sont consignées en fin de ce document.
Programme In Progress :36/182 tâches vérifiées. T174 et T030–T033 restent ouverts.

## Comportement candidat
- AutoV2 seulement, SHA source complet et posture discovery explicites.
  Le canon garde le cwd original et la clé ; Delegate direct ne déduit pas HEAD.
- La source prouve A même si HEAD B a avancé et contient du travail non commis.
  La racine Git doit rester dans le périmètre du parent attesté.
- Le worker provisionné crée la vue, y compris sur la machine du maître.
  La sonde et Prepare portent source_root et relative_cwd="." ; le mapping,
  l'époque, les digests et le parent complet restent contrôlés.
- L'import objects_only exige mapping et dépôt existants. Aucun linked checkout,
  dépôt neuf, racine additionnelle ou déplacement de HEAD/index/worktree.
  Sur le même hôte, fetch local du SHA exact, sans tunnel SSH vers soi.
- Le projecteur traduit le checkout parent vers A en discovery. L'inspection
  initiale ne lance pas de fournisseur ; la publication reconstruit les droits
  dans la vue A. Un dossier hérité seulement présent dans B refuse Prepared.
- MCP et origine négocient le support avant nouvelle opération. La vue ne
  fournit pas Git ; carte des omissions et lecteur natif de fichier existant.
- Nettoyage conservateur : aucun privilège ajouté, aucun arrêt supposé.

## Retour requis avant installation
workspace.commit_view.origin_root.v2 s'ajoute au plancher inconditionnel.
Le reçu JSON V1 ne prouve pas que ReaderA sait re-projeter un parent lié.
Qualifier et installer hors staging une image B distincte de retour AVANT
l'installation et l'activation B. Valider les rapports nonce/SHA dans les deux
sens, exercer les helpers natifs, puis écrire rollback avec l'écrivain atomique
existant sous le verrou natif. Aucun retour réel lancé pour tester.

## Tests vérifiés, sans addition des recoupements
- Linux R129 :133tests C19 PASS/1ignoré. Le cas de dossier seulementB refuse.
- Mac R129 :133tests C19 PASS/1ignoré ; transport5PASS sur R128. Les deux compléments R129 concernent le guidage et la comparaison
  de relative_cwd, désormais réservée aux vues.
- Préparation Python R129 :41PASS. Le fetch local demande A, ancêtre de B,
  conserve HEAD/index et les octets du registre, sans linked checkout créé.
- MCP R128 :30PASS Linux et30Mac. Stockage :3PASS Linux et3Mac.
- Linux :35spec168,4worktree171 et4origin_proof PASS/1recette privée ignorée.
  Droits :34PASS/7recettes privées ignorées ; le test de vue est déjà dans C19.
- Aucun de ces tests n'atteste une admission fournisseur ou une activation B.

Historique conservé : R123 a révélé l'ancienne garde d'enveloppe pour parent
lié ; exception strictement vue/discovery/base/racine ajoutée. R126 a révélé
trois assertions du jalon lecteur fermé, remplacées par les contrats de
négociation et d'absence d'effet. R127 MCP a trouvé l'ancien total11outils
alors que le lecteur fichier porte ce total à12 ; liste et présence vérifiées,
puis30tests repassés. Le nouveau test Python ancêtre manquait l'identité Git
de fixture ; ajoutée par arguments de commande comme les fixtures existantes,
sans modifier la configuration Git du compte. Aucun échec n'est compté PASS.

## Revues
2216dae5 placement,151b3d7c source liée, a502df1f droits, aaa4e210 ouverture :
résultats entiers lus et ACK/cancel demandés une fois. Détails des retenues et
réfutations dans adversarial-review-c19-creator-b.md.
Le refus initial capability_unavailable de la première revue reste de cause
inconnue ; un unique rejeu strict après vérification a admis la même demande.
Pas de cause compte, modèle ou régression ReaderA inventée.

R128 : aucune P1 ; cause de capacité de vue trompeuse retenue. R129 donne un
code de parcours de vue, sans prétendre savoir lequel des trois éléments manque.
Le conseil cite CandidateProbeV2, WorkspaceViewV1 et WorkspaceViewCreateV1,
la clé, la portée et le recours humain. Le placement hors vue reste inchangé.
L'import d'un ancêtre et la non-réécriture du registre sont maintenant testés.

## État avant activation (historique)
Revue du complément, audit de ce lot, builds et qualification du retour B,
intégration/push, distribution/activation et vérification des images chargées.
Puis qualification native de création et du lecteur omis avec Codex et Claude.
Le MCP appelant221-write ne charge pas ces nouveaux ports ; une reprise de
tour ne le remplace pas. Aucun contournement CLI/fournisseur ni restart en boucle.
Les autres catégories du programme restent à développer et vérifier.

## Gate source R129
Contre-revue finale4590229e Claude/Sonnet KS : APPROVE, aucunP1/P2 nouveau,
lecture seule du complément. Résultat3008octets lu, ACK/cancel une fois.
AuditV15 de source14/14, decision=pass, release_blocked=false, validate=true.
Archive immuable /home/moi/.local/share/bridget-audits/221-creator-b-r129.
Les notes qualité ne sont pas100 : mesures/runtime final absents et réserves
dans axis-scoring.md. Aucun succès fournisseur ni activation déduit de cette gate.

## Activation des services —31octobre2026

Source fonctionnelle8a80c257ae6fbb23ed1265b4fb8a1ea5eb43adb6 intégrée à main,
poussée et SHA distant vérifié. Build221-view-b-8a80c257 chargé ; ce constat
ne clôt pas la qualification fournisseur C19 ni le programme global.

- Cartae : maître1806269, worker1807123, mobile1806347.
- SYS : worker931194. KS : worker1358260.
- Mac : worker74018, pont74423.
- Cinq images Linux : a4a295371664762da508e53646dc22797dd90a56f078bce83057efd366de2c5c.
  GLIBC maximale2.34, construction SYS, archive source SHAab24a91cff86fb7b4e4efd81c7fd8b00c2cf780abd08738b589d78873ba068fa.
- Deux images Mac :87db459124c6b27bd202bf8d6231fa5b69fd79b05bdee88f67112a041ef70247.
- Coupure maître2,09s. Superviseurs Cartae2500061/naissance270585460 et
  SYS915861/naissance62335594 conservés. Aucun service métier relancé.
- Updaters Cartae1806351 et Mac74673 repris. Leur code Python embarqué est
  inchangé, SHA99eea133c2c2744e56adceb02caf96ae821a7df902c706a9adc99fe68dadc772,
  lu dans les arguments réels, sans copie d'environnement.

### Retours et sauvegardes
Copies B distinctes qualifiées AVANT installation du candidat et activation :
/home/moi/.local/lib/bridget-communication/221-view-b-return-8a80c257/bin/bridget
et /Users/moi/.local/lib/bridget-communication/221-view-b-return-8a80c257/bin/bridget.
Chaque image de retour a exécuté le helper natif de plan/export/lecture omise,
sans fournisseur ni worktree Git, source intacte et groupe arrêté. Le SHA réel
de chaque image correspond au candidat. Rapports nonce/SHA validés dans les
deux sens ; refus de ReaderA par le plancher B constaté.
Les pointeurs rollback Cartae et Mac visent B, écrits par atomic_json natif
sous update-host.lock, backup conservé et aucune activation concurrente.
Aucun retour réel sur une tâche n'a été lancé pour tester.

Sauvegardes :
/home/moi/.local/share/bridget-maintenance/221-creator-b-production-20261031T141516Z
/home/moi/.local/share/bridget-maintenance/221-creator-b-worker-cartae-20261031T141534Z
/home/moi/.local/share/bridget-maintenance/221-creator-b-worker-sysle2-20261031T141538Z
/home/moi/.local/share/bridget-maintenance/221-creator-b-worker-ksstore-20261031T141552Z
/Users/moi/.local/share/bridget-maintenance/221-creator-b-mac-20261031T141555Z

### Client et lien natif
UNE vérification des processus : MCP8937 reste réellement221-write-0a0d3950.
Ce client ne décrit pas commit_view ni bridget_read_commit_file. Un changement
de services ne remplace pas son image déjà chargée. Aucun restart en boucle,
changement de marqueur, d'autorité ou délégation de contournement.
UNE bridget_session puis UNE capabilities réussissent : même fil4ffa692b,
run632/révision2,45options déclarées, gel=false. Les nouveaux services et le
lien sont constatés, pas le chargement des nouveaux outils dans ce client.
Geste humain restant : quitter complètement T3 Code avec Cmd+Q, rouvrir,
puis reprendre CE MÊME FIL. Aucune compilation ou activation à recommencer.

Les deux qualifications techniques Codex/Claude sont préparées, NON envoyées,
sans task_id, dans /Users/moi/.cache/bridget221-creator-b/qualification-envelopes.json.
Elles liront A=d35452df derrière HEAD, la constante de A et un fichier omis.
Après rechargement, conserver exactement ces enveloppes ; vérifier le vrai
reçu de matérialisation, pas seulement un marqueur écrit par le modèle.
T174/T030–T033 restent ouverts ;36/182 vérifiées, pas de parité annoncée.

### Catalogue et communication
La première lecture après reprise ne voyait pas encore SYS. Lecture ciblée
ultérieure :8modèles annoncés sur chaque connexion Codex SYS et Cartae,
aucun accès payant déduit. Aucun redémarrage ou correction ajouté pour ce
constat transitoire ; cause temporelle précise non reconstruite.
Codex KS reste désactivé ; modèles Codex Mac non observés. Claude garde le
catalogue déclaré : autodécouverte non prise en charge, pas lancement interdit.
Quatre propriétaires informés par envois Bridget avant/après : monuments,
trajets bus, Politique sol et Roadie. Reçus in_flight conservés, lecture non prouvée.

### Audit source et réserves
14/14 contrôles V15 pass, release_blocked=false, validate=true.14fichiers de
code/test sont identiques au manifeste audité après intégration. Les deux
compléments après audit sont documentaires.
Qualité source, couverture de chaque axe entière dans ce seul périmètre :
sécurité65(5/5), sécurité des appels75(3/3), fiabilité56(4/4), tests65(5/5),
performance69(4/4), complexité75(3/3), qualité67(3/3), architecture69(4/4),
responsabilité75(3/3), minimalisme67(3/3), données75(3/3), traçabilité63(2/2).
Les notes ne valent pas100 ni certification globale ; détail des42critères et
limites dans /home/moi/.local/share/bridget-audits/221-creator-b-r129/axis-scoring.md
et /Users/moi/.cache/bridget221-creator-b/audit-source/axis-scoring.md.
Les neuf avis Android/Caddy restent non qualifiés pour ces assemblages ;
le signal Cargo conditionnel est réfuté sur usage inchangé, pas corrigé globalement.

### Nettoyage borné
installed appelé sur les caches Mac et SYS après installation vérifiée.
Mac : observation des processus incomplète, retrait ignoré. SYS : cache encore
détenu par /mnt/storage/bridget-build/source221-reader-a ; propriétaire conservé,
même script vérifié, installed appelé depuis cette ancre puis même refus
d'observation. Aucun marqueur de cache réécrit ni garde contournée.
Actif B, retour B, anciens exécutables sous usage (dont MCP221-write), caches,
preuves et worktrees de revue à observation incomplète conservés. Le worktree
session221 reste celui du développement en cours. Aucun retrait forcé.
Miroirs SYS/KS main et origin/main à8a80c257 ; HEAD détachés4d89cfab/16304623,
bare HEADmaster et listes de worktrees préservés. Pas de checkout déplacé.
Parcours technique : tests et reçus, pas de diaporama d'interface inventé.

Preuves privées : /Users/moi/.cache/bridget221-creator-b et
/home/moi/.cache/bridget221-creator-b sur chaque serveur.
