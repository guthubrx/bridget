# Créateur C19 — validation en cours
Source de départ : d35452df8789d61eccce54447e97fcb6f04229ba.
Le lecteur A reste actif sur les services. Le candidat R129 ouvre le créateur
dans ses binaires de test ; aucune installation ou activation B à cette date.
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

## Reste avant clôture
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
