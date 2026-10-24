# Correction — logiciel opérationnel après installation

Demande explicite : installer doit aussi démarrer ce qui est nécessaire. La
première livraison ne posait que la CLI ; ce document corrige ce périmètre.

Le script crée un service utilisateur app.cartae.bridget sur une installation
neuve, l'active et attend une réponse du daemon. Les anciennes installations qui
répondent ne sont pas interrompues. Les fédérations inaccessibles, les sockets
ambiguës et les unités étrangères sont préservées et refusées explicitement.

Référence T3 : bootService.ts et docs/user/background-service.md, lus avant
implémentation. Même usage de launchd/systemd et des sessions utilisateur.
Bridget ajoute le contrôle d'autorité distante avant création d'un maître.

## Preuves réelles

- Mac : paquet v0.4.1, HOME jetable, vrai launchctl bootstrap, daemon répondant.
- Linux Cartae : même paquet, HOME jetable et XDG_CONFIG_HOME du compte de test
  d'exécution. Vrai systemd utilisateur, unité enabled et active.
- Nouvelle session Zsh/Bash, environnement vidé : commande bridget trouvée,
  status en ligne, machine attestée, agents --json --global = [].
- Aucun fournisseur lancé, aucun secret copié, aucun redémarrage de machine.
- Défauts observés pendant l'essai : launchd créait des journaux non privés
  dans le namespace strict de la release ; ils sont désormais précréés 0600
  hors du namespace, avec Umask=077. systemd refusait un WorkingDirectory entre
  guillemets : réemploi de %h comme T3. Les deux essais réussissent après correction.
- 24 tests ciblés PASS : activation Linux et rendu plist Mac, échec de santé
  sans faux succès, refus de gestionnaire absent, de fédération en panne et
  d'un service étranger, namespace persistant et conservation des anciens tests.

Aucun reboot/logout global pour simuler l'autostart : propriétés natives,
RunAtLoad/KeepAlive et enable/is-enabled vérifiées. Les comptes Claude/Codex
restent personnels et ne sont pas implicitement créés par Bridget.
