# Consolidation dans main — 24 octobre 2026

Cette opération complète le nettoyage du même jour. L'utilisateur a demandé
le retrait des 25 anciens binaires conservés, l'examen des neuf worktrees
restants et un seul checkout propre sur main.

## Changements intégrés

- Le worktree Mac 171 contenait deux ajustements de tests encore absents :
  résolution du chemin temporaire avant construction de la fixture daemon ;
  message explicite lorsqu'une variable d'environnement inattendue atteint
  le faux fournisseur Claude. Les deux sont repris.
- Le commit f438471c contenait le diagnostic du droit d'écriture Codex.
  Son texte initial a été corrigé : il ne suppose plus qu'un autre processus
  est encore ouvert et ne promet aucune durée de libération.
  La classification exacte du refus, l'empreinte privée et le marqueur
  « tour non pilotable » sont conservés. Aucun redémarrage ni création de
  conversation de remplacement n'est ajouté.

## Décision pour chaque ancien worktree

- 171-corrections-usage-2 : les deux ajustements de tests sont intégrés.
- 163-decision-admission : aucun delta de code Bridget suivi.
  Les preuves historiques et la copie de référence .connector164 sont
  conservées hors Git ; elles ne sont pas ajoutées au produit.
- 172-codex-resume-writer et 6d9585e8 : même commit f438471c.
  Le changement utile est intégré une seule fois, avec le texte corrigé.
  Le commit d'origine reste dans un bundle Git vérifié.
- 57b6238a : le diagnostic namespace existe déjà dans b8857dfb.
  Le test actuel couvre les deux entrées native/MCP et la priorité de la
  preuve T3 invalide. Le scénario supplémentaire HOME absent de l'ancien
  test est repris dans ce test actuel, sans dupliquer son helper.
  Le reste du patch est archivé.
- 8a8f5934 : clôture du suivi après résultat distant et reprise d'entretien
  déjà intégrées en session 170. La version actuelle ajoute les contrôles
  et tests ultérieurs. Aucune ancienne variante n'est rétablie.
- b720dc9c : inventaire des worktrees déjà intégré puis renforcé en 175/176.
  Le moteur unique, les observations bornées et les erreurs explicites actuels
  sont conservés.
- c933dd39 : les changements Rust et le profil réel ont été intégrés puis
  complétés dans le chantier 171, notamment 8ac09e12. Le registre, le protocole
  et les tests transport se fusionnent sans nouveau delta. Les autres variantes
  sont comparées avec les implémentations actuelles ; aucun retour à l'ancien
  mécanisme de transfert des permissions n'est retenu.
  Le script expérimental claude_hook_closure171.py n'a aucun appelant dans
  main ; le profil réel utilise real_hook_profile171.py. Il reste dans l'archive.
- e5959986 : aucun changement local ni commit absent de main.
  La commande officielle d'annulation laisse le reçu ancien en cancelling
  avec native_cancel_unverifiable. Une lecture privilégiée de /proc confirme
  qu'aucun processus Cartae n'utilise ces espaces. Le checkout est retiré
  sur demande humaine ; aucun état de tâche n'est falsifié dans la base.

La copie T3 imbriquée dans c933dd39 est un dépôt distinct. Ses quatre fichiers
modifiés et son patch sont sauvegardés séparément. Ils ne sont pas importés
dans Bridget. Ce checkout auxiliaire et sa branche sont retirés ; les checkouts
principaux T3 et leurs modifications restent intacts.

## Nettoyage

- Les 9 worktrees secondaires Bridget sont retirés : 1 sur Mac, 8 sur Cartae.
- Leurs 9 branches locales sont retirées.
- Le checkout et la branche T3 auxiliaires imbriqués sont également retirés.
- Les 25 anciens binaires Cartae sont supprimés : 750 718 368 octets logiques.
  Le refus lié à l'observation incomplète a été levé par l'instruction explicite
  de l'utilisateur ; le binaire actif et le retour arrière récent sont préservés.
- Les fichiers modifiés sont copiés et vérifiés par SHA-256 avant restauration.
  Les fichiers non suivis et ignorés sont déplacés hors du dépôt sans perte.
  Les worktrees propres sont ensuite retirés par Git, sans --force.
- Aucun service ni fournisseur n'est redémarré. Le binaire actif n'est pas
  remplacé par cette consolidation de sources.

Les patches, snapshots, preuves et variantes remplacées sont conservés dans
les dossiers privés bridget-consolidation-20261024 sur Mac et Cartae.
Le dépôt public ne reçoit ni copie T3, ni journal brut, ni ancien build.

## Référence T3 et vérification

GitVcsDriverCore.test.ts résout les chemins réels des dépôts et worktrees :
le correctif de fixture reprend ce principe. GitVcsDriverCore.removeWorktree
utilise le retrait Git explicite ; Bridget conserve ici le retrait sans forçage
après sauvegarde et remise au propre.

CodexSessionRuntime.isRecoverableThreadResumeError ne reconnaît que les erreurs
de conversation absente. Le refus de droit d'écriture n'en fait pas partie.
Le texte Bridget ne déclenche donc aucun de ces motifs de reprise ; son propre
chemin thread/resume reste inchangé.

Cinq tests ciblés réussissent sur Cartae :
- deux tests de classification et de confidentialité du refus Codex ;
- le test des arguments, de l'environnement et du cwd du fournisseur isolé ;
- le test d'isolation worktree et de préservation du parent ;
- le test de diagnostic du namespace, enrichi du cas HOME absent.

Le premier lancement du test daemon utilisait un TMPDIR trop long pour une
socket Unix. Le même test passe avec un dossier temporaire privé court.
Ce premier échec de fixture reste dans les preuves ; aucune correction
supplémentaire du produit n'en est déduite.

La revue Bridget n'a pas été admise : keep_alive_out_of_range, puis
envelope_mismatch après correction du paramètre. Aucun verdict indépendant
n'est revendiqué pour ce lot. La comparaison et la validation sont celles
du responsable.

Preuves d'interface incomplètes : le message Codex est testé dans le transport,
mais la reprise d'une vraie conversation verrouillée n'est pas rejouée.
Aucun diaporama simulé n'est présenté comme une preuve d'usage.
Aucune suite complète, compilation release ou activation n'est revendiquée.
