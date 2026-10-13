# Livraison SPEC-161 — Client Cartae activé et ressources examinées

**Mise à jour documentaire :** 2026-10-13T09:09:27+02:00. **Développement :** Implemented,14/14. **Livraison opérationnelle :** achevée. **Git du runner :** intégré et poussé, référence distante vérifiée. **Nettoyage :** candidats traités, retraits ou conservations justifiés. **Clôture documentaire :** modifications du présent reçu encore à committer et pousser parROOT ; aucun SHA de ce futur commit n'est inventé. Le pipeline reste In Progress pour cette dernière action documentaire.

## Résultat

La commande /home/moi/.local/bin/bridget pointe depuis le13octobre2026 à08:59:38.957 CEST vers /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget. Le clientLinux communique avec le maîtreMac160. Aucune exécution distante de mission ni sélection de moteur n'est revendiquée dans161.

## Provenance et Git

- Sources compilées : e6812e3e7b2d051d630192857cc3f440180d51fa, distinctes du runner161. ROOT a revalidé les3268empreintes transférées contre le checkout160 avant son retrait.
- Commit code runner : 9368bcffd3b857a6e4271e9c613148b46dde3ab5. Runner/doc intégrés surmain :10f1e11cf04deb38b3220ce3ec1a5afae403fcef. Remote github, https://github.com/guthubrx/bridget.git ; ls-remote égal à10f1e11cf04deb38b3220ce3ec1a5afae403fcef, intégrationff-only, hooksnormaux sans contournement. Preuve git-runner161.json.
- BuildIDclient :e6812e3e7b2d, attesté par sortieCargo rustc-env, chaîne embarquée et SHA binaire/provenance. --version retourne seulement bridget0.1.3 ; status rapporte le daemon, pas l'identitéclient.
- BinaireLinux :SHA256 f008936c92086b4c440c226ac430d019dfa8ddf337dfbd11c3629bdba016e938, mode0700, ELF64x86-64. Manifest source :SHA256 cd5d6ea133e5117b7f97259f0c344a396052ed468067d1a3c68a1892cb80ae4b.
- Ancien096 conservé :/home/moi/.local/lib/bridget-communication/096/bridget, SHA25612ed07dd1dfa47ab79bf40033989cab51eaaabf9e3c21aa64237350811d24a0f.

Preuve future de clôture : /Users/moi/.cache/bridget-delivery161-20261013/git-final161.json. Ce fichier n'est pas encore une preuve au moment de cette rédaction ; ROOT doit y enregistrer le commit documentaire réellement poussé et vérifié.

## Build, activation et communication

Build réel commencé à08:52:20.906 et terminé à08:54:08.910 CEST, durée 108.004s, RC0 ; Cargo rapporte 1 min 43 s et quatre warnings hérités ; les sources160 restent inchangées. Rust1.92 explicite, source160propre, --expect-commit160, locked/offline, jobs2/nice10 et targetprivé. Le cache a été préparé depuis28archives déjà présentes surMac et7entréesindex, sans téléchargementtiers ni modificationCargo.lock. Les37objetsrelus et les sauvegardesindex restent attestés dans cache-supply-final.json.

Activation par le bloccontractuel exact, SHAe07ac087c9a58e5678be38be7004cf9fb045d786ddd386223062b26aaf8674d9, RC0. Readlink après activation confirme161. Status/who ont chacun RC0 pour le candidat puis l'alias. Aucun avertissement de divergence de version. who produit project_unknown car aucun projet n'est fourni : cela ne prouve pas un annuaire projet, et reste une limite explicite.

RecettePython095 distincte, RC0 à08:59:59 CEST :5événements, deuxagents enregistrés, mêmeannuaire, deuxmessages livrés dans les deuxsens, deuxagents stopped à la fin. Elle atteste le transport et le maître ; elle ne lance pas le binaireclient.

- Agents :679ab00b-932a-420f-8955-7db39da90118 et9c0ce73a-f990-4602-b58f-747195cbc310.
- Messages :2fba926e-30a3-46d9-9db2-609bf48da346 etf32eca80-9d29-4efd-99e4-6dded31052b0.
- Journal :/Users/moi/.cache/bridget-delivery161-20261013/roundtrip161-after.jsonl, SHA256f3dd582a7836a52d42f5227c646a8b78443f798d4c27f393218ff717094e995d.

Écritures limitées aux inscriptions/messages normaux de recette ; aucune édition directe, migration ou suppression de base. Aucun retour arrière réel nécessaire. Le retour096 est conservé et son mécanismealler/retour a été testé enfixture ; aucune restauration deproduction exécutée n'est inventée.

## Processus et runtime préservés

runtime161-final.json confirme inchangés lesPID/naissances : maître20937,13oct00:25:51 ; pont21097,00:25:54 ; T360116 et60228,11oct01:13:35/36 ; tunnelSSH34764,12oct17:13:12. Aucun redémarrage. AucunprocessusBridgetCartae avant/après ; pgrepRC1 signifieabsence. Inventairecore identique avant/après candidat : socket,federation.env,tmp déjà présents. Aucun nouveauTMPcore observé.

## Tests, convergence et audit

Développement14/14 vérifié avantcommit. Harness complet unique30PASS/1FAILhistorique ; diagnosticPythonHOME, correction-B puis8rejeuxaffectésPASS. Les23autres cas gardent leurs résultats antérieurs ; aucune secondeFULL, aucun GREENglobalRust. TestpaquetRC0 et fixtureatomique du paquet1PASS après correctif d'embarquement du contrat.

Converge P1 puis P2 :CONVERGED surdéveloppement/procédures, aucune tâche ajoutée, tâchesSHAa3b40343487f5dcb4fc84eee823d8ff4d557b6ea110e9c79a7acad101fa079db inchangé pendant cesphases. Les preuves réellesG8 sont venues ensuite ; ce reçu ne transforme pasConverge enrecetteproduction.

ClaudeCodeR1 APPROVE_WITH_CHANGES :deuxremarquesdocumentaires résolues, pasdeblocagecode. AuditfinalA100 sur sept fichiers / 1169 lignes seulement, aucunfindingfinal, validateurRC0. Aucun auditglobal du dépôt ou du client160 revendiqué.26fichiersdepreuve copiés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-10-13/session-2026-10-13-spec-161-01 et SHA relus à cette rédaction.

## Nettoyage exécuté et conservations

| Ressource | Résultat et raison |
|---|---|
| /private/tmp/b89-package-test.ljfkhJ | Retiré après archivageSHA de7preuves ;744fichiers,24 668 206octets logiques. AucuneDB, travauxétrangers préservés. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae | Retiré parROOT sansforce après transfertpreuves et contrôlesusage/propreté/fusion. Chemindésormaishistorique. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-source160 | Checkoutdétaché retiré parROOT sansforce après comparaisonmanifest ; cheminhistorique. |
| session-161-client-cartae | Branche retirée sansforce aprèsfusion/push. |
| /home/moi/.local/lib/bridget-communication/096/bridget | Conservé, rollback utile vérifié. |
| /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/bin/bridget | Conservé, clientactif. |
| /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/source/target |348 420 473octets logiques conservés : nettoyageprojet refusé carlsof incomplet, mêmeavecsudo. Garde noncontournée. |
| /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d/source |90 294 080 octets logiques hors target déjà compté, conservés avec le préfixe actif pour provenance. Somme source + target : 438 714 553 octets logiques, pas une mesure physique ; aucune purge affirmée. |
| /home/moi/.cargo/registry | Cacheutile partagé conservé ;37objetsSHA de préparation attestés, aucune purgeglobale. |
| /Users/moi/.cache/bridget-delivery161-20261013/cache-index-backups | Sauvegardesindex et preuves conservées pourretourarrière. |
| /Users/moi/.cache/bridget-delivery161-20261013 | Preuvesdurables et historiqueconservés ; aucune preuve utile supprimée. |

Les simulations build.py clean --dry-run ont chacune RC1 : «lsof indisponible ou incomplet ; nettoyage ignoré». Le diagnostic lsof prévient qu'il ne peut passtat fuse.portal /run/user/1002/doc, donc son inventaire peut être incomplet. Aucun nettoyage réel du target n'est affirmé. Cette conservation justifiée termine l'examenG9 prévu par le contrat ; elle n'est pas une purge réussie. Aucunpréfixe-r2 créé ; manifesttemporaire retiré partrap. Aucun gain physique APFS ou disque distant déduit des tailleslogiques.

## Critères, temps et suite

SC001 provenance, SC002 communicationdistincte, SC003 servicespréservés, SC004 gardes/fixture/rollbackconservé, SC005 provenanceGit/opérationnelle et décisionscleanup, SC006 contrôlesciblés ont leurs preuves ci-dessus. La seuleaction restante est commit/push/vérification des métadonnées de clôture. Pas de nouvelle tâche de code ; aucune case noncochée.

ETA161 annoncée120–240min à08:23, puis maintenue après14tâches. Débutpipeline relevé08:02 ; activation à08:59:38, soit57min39s depuis le début. À cette rédaction 09:09:27 CEST :environ67.5min depuis08:02, soit-43.8% par rapport à la borne basse120min et-71.9% par rapport à la borne haute240min. C'est un point intermédiaire avant le commit final, pas un temps de clôture Git. Cause principale de réduction :cache local réemployé, compilation Linux en 1 min 43 s et vérifications ciblées sans répétitionFULL.

## Registre des empreintes relues

| Preuve privée | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery161-20261013/git-runner161.json | ccb0f595f68f633ec58049df4f9fc3bcdfa4f7dfbc02c0047750803e471f9cf8 |
| /Users/moi/.cache/bridget-delivery161-20261013/build161-result.json | 6982d63859c014b4b91be1767d7fc585088554892612b47f28b152a3952fe193 |
| /Users/moi/.cache/bridget-delivery161-20261013/client-build-identity.json | bd3731fb7d3a417a9dafc308028bcf3de7b0bb74e985a2d8a8bdb61ddd565ecb |
| /Users/moi/.cache/bridget-delivery161-20261013/SOURCE-MANIFEST161.sha256 | cd5d6ea133e5117b7f97259f0c344a396052ed468067d1a3c68a1892cb80ae4b |
| /Users/moi/.cache/bridget-delivery161-20261013/activation161.json | 1c69680e9691d95a950527e49313237f795d08b17194e648b3ff323906d67958 |
| /Users/moi/.cache/bridget-delivery161-20261013/active161-checks.json | 97d68246757d20ed88d265b039af7cc314b07595801e5b4502eab21f73cf1c99 |
| /Users/moi/.cache/bridget-delivery161-20261013/roundtrip161-after.jsonl | f3dd582a7836a52d42f5227c646a8b78443f798d4c27f393218ff717094e995d |
| /Users/moi/.cache/bridget-delivery161-20261013/runtime161-final.json | b0478577ecea393f4545c286cfa350c7140fe49683017e7f8f310c69fe78f10f |
| /Users/moi/.cache/bridget-delivery161-20261013/cleanup161-package.json | bffaaedd8acccf058a25e5cb8ff0f2a034d1a1dd3372599dece5ed5d9a9db8f1 |
| /Users/moi/.cache/bridget-delivery161-20261013/cleanup161-retained-cache.json | 7d76c03ce42421d9921abf0fa16d10d2bf3e6dcdd0cc1c16ad4adba54b4ee22b |
| /Users/moi/.cache/bridget-delivery161-20261013/cleanup161-remote-dry-run.json | 20e4d06bfe827c7c7f3b61db5684f2f9acdf599acb8600f4863bf04a99cc8057 |
| /Users/moi/.cache/bridget-delivery161-20261013/cleanup161-remote-root-dry-run.json | b61fdca27e72eff4d14b5ef6ac7aa3c24be4eb52b08cc6a02f5aaaa65ddb07db |
| /Users/moi/.cache/bridget-delivery161-20261013/audit-archive161.json | 7e91f1b4142aa909de9cee5fc8844dd958c05edd158b17372f49e6070bbd28fb |
