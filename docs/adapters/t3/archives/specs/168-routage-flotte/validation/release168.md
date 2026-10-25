# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/release168.md](../../../../../../../specs/168-routage-flotte/validation/release168.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les parents, daemons, relais et tunnels privés ont été arrêtés selon les reçus de nettoyage et la clôture ROOT. Cette publication n’effectue aucune action de service. La production B était déjà installée et active. T3 n’est pas relancé ; le serveur T3 97328 est un changement externe signalé par ROOT.

<a id="extrait-2"></a>

## Extrait 2

FULL unique antérieure au correctif B : 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des échecs initiaux PASS au dernier replay, trois annulations159 non validables sur Linux. La cause initiale T3 reste inconnue. Aucun GREEN global. Audit A V15 r2 PASS et delta T045 documenté restent distingués ; pas de nouveau scan, FULL ou passage Converge. Deux LOW RUSTSEC-2026-0098 et RUSTSEC-2026-0099 sur rustls-webpki0.101.7, et un INFO maintenance RUSTSEC-2024-0449 sur unsafe-libyaml0.2.11 restent conservés.
