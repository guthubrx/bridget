# Plan

1. Script POSIX autonome, compatible avec `curl | sh`, dépendances minimales
   curl/tar/SHA-256 pour les paquets. Aucun gestionnaire de paquets système.
2. Sources : Git, Python >= 3.11 et compilateur C requis ; amorçage Rust officiel
   si absent, toolchain du dépôt, Cargo.lock et wrapper build.py existants.
3. Validation en HOME jetable : vraie archive, corruption, reprise et erreurs.
   Construction réelle sur Cartae, sans modifier son installation active.
4. Documentation EN/FR, push et publication du même script sur cartae.app.

T3 consulté : docs/user/install.md, scripts/release-smoke.ts et
.github/workflows/release.yml dans /Users/moi/11.Repositories/t3code.
Réemploi : distinction paquet publié/code de développement et essais de paquet
isolés. T3 utilise npm/Electron ; Bridget est un exécutable Rust, donc archive
native et compilation par le wrapper existant. Aucun nouveau moteur de service.
