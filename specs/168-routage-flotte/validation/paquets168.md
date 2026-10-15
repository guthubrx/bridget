# Paquets168 — version0.4.0

## État courant — 15/10/2026 à 17:13 CEST

**In Progress, 53/56 tâches cochées par ROOT.** T045 et T046 sont closes. T047, commit documentaire C1, est en cours ; T048 et T049 restent ouvertes. Le nettoyage final de la source appartient à ROOT. Aucun statut Livré ou Implemented n’est annoncé.

La source binaire B `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0, est construite et activée sur Mac/Cartae. Les trois voies réelles ont résultats et rejeux vérifiés : Laya `decision_service` sur 2 candidats, Haiku `lightweight_model` sur 21 candidats, dernier repli `deterministic` sur 21 candidats. Détail : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md.

**Release publiée et vérifiée :** https://github.com/guthubrx/bridget/releases/tag/v0.4.0 . Tag annoté `43202c9de66ce466b76ba05bd3077699ac88ca3d`, cible exacte B ; quatre actifs retéléchargés et hashes conformes. Reçu : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/release168.md, SHA-256 `15a7b44610ebe060c47164e6c511606e67c37c4d2e4e85a71c2db5be879b9f5d`. Preuve privée : /home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication168.json, SHA-256 `d815ca75ca354ec4941577cb39a29aab62cf641ac71895f2ba69e3f0c8ea9666`.

Santé finale reçue : B en ligne, catalogue 27, pont 32 fils vers 97328 ; DB readonly quick_check ok/FK0, versions 7/12 ; worker B 3767260 actif, configuration inchangée. Application T3 65544 et tunnel 60206 préservés ; le serveur 65608 a été remplacé extérieurement par 97328 né à 16:51:02 CEST, sans signal T3 de notre part. La restriction temporaire de 16:56 CEST est levée.

Preuves existantes :

- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 `be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba` ;
- /Users/moi/.cache/bridget-delivery168/final-health/database-health.json — SHA-256 `98feb24442412b8e2ce12d5c6a92e75605c500bd6f67b4e7fc349fc6cebab147` ;
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 `a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a`.

Limites : FULL unique 2820 PASS/32 FAIL/71 ignorés ; 29 échecs initiaux PASS après reprises, trois annulations 159 non validables sur Linux, cause initiale T3 inconnue malgré replay PASS. Deux LOW rustls-webpki et une INFO unsafe-libyaml restent documentés ; aucun GREEN global. Aucun nouvel audit/FULL/P4/build/service/test/modèle par cette rédaction. Aucun retour direct au binaire 0.3.0 après reçu admission v2. Les commits documentaires descendants conservent la source binaire B et ne nécessitent aucune reconstruction.

Cet état daté prévaut sur les checkpoints historiques ci-dessous, dont les anciennes mentions de publication en préparation. Les DB, preuves et sauvegardes sont protégées.

---

> État courant : source B 431b78f72bdd73eac4772af17bf36879d7a629d2 ; checkpoints A historiques.
> Référence : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/paquets168.md (section « Checkpoint B courant »).

Source commune :269de64e172f71f8856373fe8b09cb2d8642da52. Les deux constructions partent de l’export Git propre3433fichiers, sans utiliser le worktree documentaire modifié ensuite.
Commande réemployée : nice -n10 python3.12 scripts/build.py cargo build --release --locked --offline --jobs 2 -p bridget-daemon --bin bridget. Aucune suite de tests rejouée pour ce build.
Linux : RC0,89,918s ; bridget0.4.0 ; BUILD_ID complet embarqué ; aucun compilateur restant. BinaireSHA256 a03e86f84851f81d082bc4c70f5fbd53f63c31d65482f1e785a43e05f3c53887.
Mac : RC0,126,022s ; bridget0.4.0 ; BUILD_ID complet embarqué ; codesignadhoc et vérification stricteRC0 ; aucun compilateur restant. Binaire signéSHA256 17ec685eb9c80be7e91dc5350119f0404cbf8ddf57fe1e5e8710d00e5d3d6b69.
Archive Linux : /home/moi/.local/state/bridget-delivery168/build/runs/0.4.0-269de64e172f-linux/bridget-0.4.0-linux-x86_64.tar.gz
SHA256 archiveLinux :1c67b9e2311153fd51a6f1e2204b222b02a2a1e85ea40db736f9fdb85d636742.
Archive Mac : /Users/moi/.cache/bridget-release168/0.4.0-269de64e172f-mac/bridget-0.4.0-darwin-arm64.tar.gz
SHA256 archiveMac :64284f5c0c354ee8453f7060b06826cfc930276a6fc3be476eb9d59e2bdf0b11.
Chaque archive contient le binaire, licence, READMEfr/en, changelog, guide d’installation, RELEASE.txt et SHA256SUMS ; contenu et sommes internes vérifiés par le constructeur revu, puis hashes des binaires/archives recoupés par ROOT.
Reçu Linux : /home/moi/.local/state/bridget-delivery168/build/runs/0.4.0-269de64e172f-linux/build-receipt.json — SHA256 0c4fc779d2452d7c475d5d37c44dbc1d106ec97665d38e8b349c2099ed72df3c.
Reçu Mac : /Users/moi/.cache/bridget-release168/0.4.0-269de64e172f-mac/build-receipt.json — SHA256 cbfafb8b4324489a3f0f580455d6f3f1746f6b157c1fd7721e0ffdfa0ae2f1b9.
Entrées : archive sourceSHA256 e8036348d3e543cee260a5dc63d2e206280d5da4056f49af8e66316d53f83ab0 ; manifeste exportSHA256 9ca7fe06873edd6c9ffcf8998d35891d522b7449cb4474f6c4e0a114996bd232 ; constructeur privéSHA256 86d5ed620fb83b679b00c1aff765bfbc9fb16fcc1faa827b30bd2f5bb6f1c069. Sources exportées et paquet source inchangés pendant les builds ; leur manifeste communSHA256 c324b45eb1f44745baba699629d374fd404134b77936a950117e8163c23cffa0.
Les journaux contiennent des avertissements de visibilité/code mort ; ils sont conservés complets, sans succès global du workspace déduit. Les trois limitesLinux159 et la cause initialeT3 inconnue restent celles de T036.
Ces paquets sont des candidats. Les deux installations actives0.3.0 et leurs liens sont restés inchangés pendant les constructions. Intégration, activation, recettes réelles et publication restent distinctes.


## Checkpoint B courant — paquets de la source active

Les paquets A décrits plus haut sont historiques. Source commune courante : `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0. Les deux builds B partent du même export détaché propre et conservent leurs sources inchangées. Manifeste de paquet source commun SHA-256 : `23088ff43c90289f511827466e641855321adc798c950343bab7228ac3c540fc` ; archive source SHA-256 : `104ba544cdb8ba41ae0862887b9009fa5b3664eb3965c7eb3bf27e8c18211e9c` ; manifeste d’export : `51cb919dd8bebffa33a449f9e4febcc085a64ea5914cb3d9509792eda8796a05`.

- macOS : RC0 en 161,838 s ; binaire bridget 0.4.0, BUILD_ID complet B présent, signature vérifiée. Binaire SHA-256 `3502daf850bae303a9ad0f4bdd2dbc073f6b36fdd51ced8ac62f52014aa15bed`.
- Linux : RC0 en 93,996 s ; binaire bridget 0.4.0, BUILD_ID complet B présent. Binaire SHA-256 `3b86b8fd48698e567d5f190a67f946d84d99fecd4526441ba6702bba9c3bf95a`. Aucune signature macOS n’est attribuée au paquet Linux.

Reçus de build :

- /Users/moi/.cache/bridget-release168/0.4.0-431b78f72bdd-mac/build-receipt.json ; SHA-256 `6c9b38f7a80238fb21d4160bd826cd34ba5dd7f038a8b4629a2215af26a45ffc`.
- /home/moi/.local/state/bridget-delivery168/build/runs/0.4.0-431b78f72bdd-linux/build-receipt.json ; SHA-256 `6206708987cb9b4707de2844dd6786d0f67b91f8b20d8ef6e59ac2e941188fc2`.

Archives B et fichiers de sommes conservés :

- /Users/moi/.cache/bridget-release168/0.4.0-431b78f72bdd-mac/bridget-0.4.0-darwin-arm64.tar.gz ; SHA-256 `3b57805bd4a2b95dedab14e950118d698166c06b2dc0f2085a5f7d799242d687`.
- /Users/moi/.cache/bridget-release168/0.4.0-431b78f72bdd-mac/bridget-0.4.0-darwin-arm64.tar.gz.sha256
- /home/moi/.local/state/bridget-delivery168/build/runs/0.4.0-431b78f72bdd-linux/bridget-0.4.0-linux-x86_64.tar.gz ; SHA-256 `ae8b7c17c1abea84a39fd01385961a8daaad7f803f5fea87f8fb0ed64dd30aa2`.
- /home/moi/.local/state/bridget-delivery168/build/runs/0.4.0-431b78f72bdd-linux/bridget-0.4.0-linux-x86_64.tar.gz.sha256

Les reçus détaillent les huit entrées de chaque archive et leurs SHA : binaire, notes RELEASE, SHA256SUMS, licence et documents d’installation. Journaux non tronqués, aucun compilateur restant au contrôle des builds. Les champs active_installation=false, services_changed=false et published=false décrivent uniquement ces builds détachés : l’activation exécutée ensuite est prouvée séparément dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md. Aucun test n’a été lancé par les constructions. État ROOT au présent mandat : 51/56, sans cochage par cette rédaction. Les recettes réelles T045 sur B sont en cours ; la release n’est pas publiée. La session reste In Progress. Les limites de T036 sont conservées : FULL unique 2 820 PASS / 32 FAIL / 71 ignorés ; 29 des 32 échecs initiaux PASS après reprise ; trois annulations 159 non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. Aucun GREEN global.
