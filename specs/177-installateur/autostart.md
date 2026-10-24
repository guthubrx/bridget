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
- 26 tests ciblés PASS : activation Linux et rendu plist Mac, échec de santé
  sans faux succès, refus de gestionnaire absent, de fédération en panne et
  d'un service étranger, namespace persistant et conservation des anciens tests.
- Réinstallation réelle Mac : même PID du daemon, aucune interruption.

## Revue

Task 1f5e4450-052d-4960-b361-e83bed0313f0, clé
bridget177-autostart-review-20261024-01 : APPROVE_WITH_CHANGES.
Remarques confirmées et corrigées : distinguer une unité créée ici d'une unité
préexistante lors d'un échec, signaler un arrêt impossible à confirmer. Deux
tests couvrent ces cas. Message explicite sur le démarrage automatique inchangé
d'une installation existante. Un daemon manuel n'est pas converti en service.

La recommandation de délai externe n'est pas retenue : get_status_until dans la
release v0.4.1 et le code courant partage un délai de 10 s, avec preuve d'identité
bornée à 2 s. Pas de dépendance gtimeout pour Mac. La boucle est bornée à cinq
sondes et cinq attentes d'une seconde, pas annoncée comme un délai global de 5 s.
Le refus d'une socket existante dit désormais « impossible à vérifier avec cette
version », sans conclure à tort que le daemon ne répond pas.

L'essai MacBook reste distinct : hôte Bonjour identifié et clé d'hôte retrouvée
dans les clés connues ; authentification SSH du compte moi refusée. Aucun
installateur exécuté sur ce MacBook, aucun droit changé.

Aucun reboot/logout global pour simuler l'autostart : propriétés natives,
RunAtLoad/KeepAlive et enable/is-enabled vérifiées. Les comptes Claude/Codex
restent personnels et ne sont pas implicitement créés par Bridget.
