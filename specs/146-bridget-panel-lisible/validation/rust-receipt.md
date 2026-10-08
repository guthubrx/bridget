# Preuve T001/T002 — Rust 146

Date : 2026-10-08. Logs fournis par le responsable Rust et relus par le responsable documentaire. Ce dernier n'a pas relancé les tests.

Cache privé : `/Users/moi/.cache/bridget-build-146.EFWCYG`.

RED réel sur les méthodes du socle145 avant produit146 :2 tests échouent, exit101. La liste retourne UUID…001 au lieu du fil récent…002 ; l'historique commence à séquence1 au lieu de7. Preuve : `/Users/moi/.cache/bridget-build-146.EFWCYG/red-146.log`.

GREEN :14 nouveaux tests146 uniques, répartis9 lib,1 protocole,4 CLI. Logs : `/Users/moi/.cache/bridget-build-146.EFWCYG/green-unit-146.log`, `/Users/moi/.cache/bridget-build-146.EFWCYG/green-protocol-146.log`, `/Users/moi/.cache/bridget-build-146.EFWCYG/green-cli-146.log`.

Les tests lib couvrent liste globale récente, date du dernier message, UUID et vide/fermé, historique DESC borné en nombre et octets, snapshots de corrections, trous de séquence, négociation singleton et les deux gardes sans maintenance. La bienvenue réelle ne négocie que `human_thread_view_recent_v1`. Le plan SQLite utilise les indexes membres, fil et entrée ; un tri temporaire est attendu. Aucun benchmark de production n'est déduit du plan de requête.

Régressions :13 tests filtre145 et15 filtrethreads passent. Ces filtres se recouvrent et contiennent aussi des tests146 ; ils ne s'additionnent pas en un total unique. Quatre CLI 145 passent en plus. Logs : `/Users/moi/.cache/bridget-build-146.EFWCYG/regression-145.log`, `/Users/moi/.cache/bridget-build-146.EFWCYG/regression-threads.log` et le log CLI ci-dessus.

Fmt/diff exit0 rapportés. Les corrections de fixtures mandatory/AlreadyNegotiated/permissions privées sont déclarées ; elles n'assouplissent pas le produit. La compilation release normale est en finalisation de reçu ; aucun artefact installé, aucun hash final de release et aucun succès production ne sont revendiqués ici.
