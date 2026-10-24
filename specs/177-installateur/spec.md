# 177 — Installer Bridget en une commande

Statut : Implemented — livré le 24 octobre 2026. Demande validée.

Installer ou mettre à jour Bridget sur Linux et macOS avec une commande publique.
Par défaut, prendre la dernière release GitHub, figée et déjà compilée.
Avec `--source`, construire le commit courant de `main`, identifié précisément.
Le DMG et l'installation sur SYS-LE-2 / KS-STOR sont exclus de cette livraison.

Critères : téléchargement HTTPS, empreinte SHA-256 obligatoire pour la release,
refus d'une plateforme sans paquet (aucun repli implicite vers les sources),
validation du candidat avant remplacement atomique, binaire précédent conservé,
installation utilisateur sans sudo, PATH préparé sans modifier les comptes.
Les données, profils fournisseurs et services actifs restent intacts.
L'installation du programme ne vaut pas rattachement à un maître Bridget.

Commandes publiques : `https://cartae.app/bridget/install.sh` et copie GitHub.
Une version précise doit pouvoir être choisie avec `--version vX.Y.Z`.
Les échecs de téléchargement, contrôle et compilation conservent le binaire installé.
