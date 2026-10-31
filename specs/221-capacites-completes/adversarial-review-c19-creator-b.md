# Revue C19 créateur B — raccord source et placement

Revue de conception2216dae5 (Claude/Sonnet KS), résultat entier lu6554octets,
ACK/cancel demandés une fois. Source d35452df ; lecture seule, pas de test.
- Observation locale exclue pour une vue : retenu, branche worker conservée.
- Mapping absent avant choix et Delegate sans SHA : retenus, refus explicites.
- Préparation locale à qualifier : reste dans la recette de création publique.
  Le worker exige le commit exact avant sonde ; aucune garantie depuis le HEAD.

Revue R121151b3d7c (Claude/Sonnet KS), résultat entier lu8270octets,
ACK/cancel demandés une fois. Snapshot R121 ; aucun test, build ou activation.

| Objection | Vérification | Décision |
|---|---|---|
| Source relative d'un worktree absente après objects_only | mapping_paths, native_view_plan et réserve utilisent remote_root+relative_cwd | Retenue : source réduite à la racine de A côté sonde/Prepare, canon original conservé. |
| Réduction maître seule suffisante | project_remote_permissions exige encore le chemin du cwd parent sur la cible | Incomplète : alias de cwd dans le projecteur et preuve de racine Git incluse dans le parent nécessaires. |
| Ancien script cible ignore objects_only | ssh dans prepare-project.py transmet les octets du script courant via base64 ; elle n'appelle pas un script installé sur la cible | Réfutée pour ce transport ; pas de champ d'écho ajouté sans besoin. La capacité source et le contrôle avant trame restent requis. |
| Disparition après inspect crée un checkout | branche else de remote_receive peut créer une destination | Retenue : garde fermée dans la branche de création et test de renommage concurrent sans perte de données. |
| Drapeau global fermé accusé comme capacité source manquante | exchange testait les deux causes dans la même condition | Retenue : workspace_commit_view_not_enabled distinct de origin_commit_view_unsupported. |
| project_mapping_unknown sans conseil | admission_guidance211 possède déjà le message et sa décision ciblée ; observe conserve mapping_refusal | Réfutée sur cette tête. Le mode objects_only refuse un mapping neuf avant création. |
| Aucun nouveau plancher nécessaire car JSON inchangé | recheck_view_file_actor ReaderA reconstruit readiness_at avec le parent lié sans alias | Réfutée pour la lecture des fichiers omis : nouveau plancher origin_root.v2 et retour B à qualifier avant ouverture. |

Le besoin reste complet. Aucun travail métier ni profil, compte ou modèle
n'est remplacé. Les tests de C19 se recouvrent et ne sont pas additionnés.
La suite doit vérifier les droits de lecture, les deux constructeurs, la
recette depuis source liée, l'ancien lecteur et le retour B réel.


## R124 a502df1f — résultat8644octets lu, ACK/cancel une fois
Aucune P1 ; APPROVE_WITH_CHANGES, lecture seule du patch R124.
- P2 alias vers B : l'inspection atteste le dépôt avant export, sans fournisseur.
  poll_prepared_view recharge ensuite le mapping vers la vue, puis reconstruit
  la politique avant publication. Test réel de pipeline ajouté : notes non
  suivi dans B, droit hérité vers notes, vue A sans notes ; refus de permissions,
  aucune publication de Prepared, source arrêtée, B inchangé. PASS R126/R127.
  La garantie d'invisibilité OS générale reste explicitement exclue du contrat.
- P2 plancher sans drapeau : décision conservée, car désactiver la création ne
  retire pas les reçus. Qualification et installation du retour B obligatoire
  AVANT installation/activation, pas seulement avant première admission.
  Le validateur installé est bien appelé par scripts/update.py (storage_check).
  Aucun retour A n'est déclaré compatible. Le pointeur sera écrit atomiquement.
- P3 sourcefacts sans garde : faits de lecture distincts de l'admission ; pas
  de Task, de jeton ou de copie implicite depuis ces ports. Les admissions et
  préparations restent négociées et contrôlées. Pas de garde redondante ajoutée.
- P3 parent.cwd avec ParentDir : worker_absolute_path ajouté uniquement à
  l'exception vue ; contre-test explicite. Anciennes formes inchangées.
- P3 cause capacités : cause renommée origin_view_capabilities_missing,
  message demandant lecteur et créateur ; aucun manque unique inventé.
- P3 validate_routing_source sans usage produit : réservé cfg(test) ; les
  appels produits utilisent validate_parent_source_base avec le parent réel.

Le candidat R127 active désormais le créateur dans ses seuls binaires de test.
Les processus installés n'ont pas changé. Revue de l'ouverture et de la preuve
complète maître/worker encore requise avant livraison.

## R128 aaa4e210 — résultat6239octets lu, ACK/cancel une fois
Lecture du patch complet, sans shell, tests ni activation ; APPROVE_WITH_CHANGES.
- P2 guidage : confirmé, CapabilityRequired sur une vue était nommé comme
  absence de sonde V2. Code worker_commit_view_capabilities_unsupported ajouté.
  La cause reste capacités de vue non confirmées ; les trois capacités sont
  citées, sans inventer laquelle manque si le refus ne la distingue pas.
- P2 comparaison relative_cwd hors vue : modification limitée à CommitView.
  Shared/Worktree gardent leur comparaison historique. Régressions168/171 PASS.
- P2 fetch ancien SHA : test existant renforcé, A ancêtre de B demandé localement,
  contenu de A lu, HEAD/index et registre inchangés ;41tests Python PASS.
- P3 publish_mapping : aucune écriture si la clé existe et vaut le mapping
  attendu ; vérification des octets du registre dans le cas nominal renforcé.
- P3 mutex : validate_routing_parent_source sort de son bloc Mutex avant
  sources(...) et validate_parent_source_base. Le commentaire reste exact.
- P3 enum de mode : non retenu. Deux valeurs, argument interne uniquement,
  mode vue explicite aux quatre appels ; un type partagé n'ajoute aucun contrat.
  Les branches existantes de sélection/publication gardent leur responsabilité.
- P3 imports : à vérifier avec la construction non-test du candidat.
La comparaison actuelle et le guidage sont le seul complément produit R129.
Pas de nouvelle autorité, store, dépendance ou opération fournisseur.

## R1294590229e — résultat3008octets lu, ACK/cancel une fois
APPROVE, aucune P1/P2 nouvelle. Fermeture guidage/corrélation/import ancêtre
confirmée par lecture du delta et extraits. Aucun test ou runtime attesté par
le relecteur. Son attente Mac est maintenant levée par133tests R129 PASS.
