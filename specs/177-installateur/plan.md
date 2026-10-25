# Plan

1. Script POSIX autonome, compatible avec `curl | sh`, dépendances minimales
   curl/tar/SHA-256 pour les paquets. Aucun gestionnaire de paquets système.
2. Sources : Git, Python >= 3.11 et compilateur C requis ; amorçage Rust officiel
   si absent, toolchain du dépôt, Cargo.lock et wrapper build.py existants.
3. Validation en HOME jetable : vraie archive, corruption, reprise et erreurs.
   Construction réelle sur Cartae, sans modifier son installation active.
4. Documentation EN/FR, push et redirection cartae.app vers le script GitHub.

Correction de démarrage : reprendre le mécanisme de
apps/server/src/cloud/bootService.ts (renderBootServiceUnit, launchdManager et
systemdManager) : service utilisateur natif, PATH explicite, bootstrap sans
kickstart forcé, WorkingDirectory=%h pour systemd, échappement XML et %%.
Différences : préserver les connexions Bridget déjà actives et refuser une
fédération inaccessible au lieu de créer une deuxième autorité. La réponse
`agents --json --global` atteste la connexion et l'inventaire réel du daemon.
Pas de nouveau service maison, aucun changement Rust ni migration de données.
