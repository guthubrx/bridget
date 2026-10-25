# Archives techniques de l’adaptateur T3

Document d’origine : [specs/176-observation-bornee/validation/consolidation-main-20261024.md](../../../../../../../specs/176-observation-bornee/validation/consolidation-main-20261024.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

La copie T3 imbriquée dans c933dd39 est un dépôt distinct. Ses quatre fichiers
modifiés et son patch sont sauvegardés séparément. Ils ne sont pas importés
dans Bridget. Ce checkout auxiliaire et sa branche sont retirés ; les checkouts
principaux T3 et leurs modifications restent intacts.

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

Les patches, snapshots, preuves et variantes remplacées sont conservés dans
les dossiers privés bridget-consolidation-20261024 sur Mac et Cartae.
Le dépôt public ne reçoit ni copie T3, ni journal brut, ni ancien build.
