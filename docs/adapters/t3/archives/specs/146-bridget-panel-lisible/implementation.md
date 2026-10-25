# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/implementation.md](../../../../../../specs/146-bridget-panel-lisible/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T008 terminée sur recette DOM réelle isolée du principal : 137 messages uniques en ordre descendant, snapshot 137 et panneau de 360 px sans débordement. Aperçu de quatre lignes, dépliage, copie exacte de 1746 octets, clavier et focus sont vérifiés. Les refus et contextes périmés purgent les articles. Recherche du corps replié vérifiée sans requête supplémentaire. Preuve `/Users/moi/.cache/t3-spec146-preview.650MHJ/browser-receipt146.json`. Captures T3 échouées : aucune image prétendue.

<a id="extrait-2"></a>

## Extrait 2

Après la clôture du développement, l'utilisateur demande explicitement : « installe, committe, fusionne ou pousse ne relance pas ». Cette autorisation couvre l'enregistrement, la fusion et le push du socle145 nécessaire puis du delta146, ainsi que l'installation sur disque. Elle ne couvre aucun redémarrage de T3, de Bridget ou des boucles. Les modifications de compétences142 présentes dans la racine restent hors livraison et sont conservées. Aucun worktree n'est supprimé.

<a id="extrait-3"></a>

## Extrait 3

Socle145 enregistré séparément : Bridget d40324fd, T3 d381302ec1. Delta146 : Bridget18c7fe20, T3 0bd1e7a52ced2d03f3e71e7dc7569ac16e1927a3. Documentation générale : b808a52e6f4471e33f07bd9e4b16c263d7a18e48. Fusion rapide sans conflit, puis push confirmés vers github/main pour Bridget et fork/local/v0.0.45 pour T3. Les deux PR sont liées au fil T3 et réellement MERGED : https://github.com/guthubrx/bridget/pull/1 et https://github.com/guthubrx/t3code/pull/1. Aucun push vers le dépôt T3 amont.

<a id="extrait-4"></a>

## Extrait 4

Reprise fraîche :603 tests T3 PASS,14 tests Rust146 uniques PASS (9 unitaires,4 CLI et1 protocole). Les quatre tests CLI sont aussi relus contre le binaire release exact. Les premiers essais de socket Rust ont rencontré la limite de chemin macOS ; TMPDIR=/tmp a permis la reprise complète sans changement de source. Builds web, serveur et desktop privés PASS. Signature ad hoc puis vérification profonde stricte PASS. Vérificateur du paquet :2291 PASS/0 FAIL,2223 fichiers web comparés au build frais et45 fichiers natifs. Reçu : /Users/moi/.cache/t3-spec146-package.QwWpZB/delivery-build-receipt146.json. Les empreintes gelées correspondent au code committé et au build.

<a id="extrait-5"></a>

## Extrait 5

Application remplacée au même emplacement /Applications/T3 Code (Local).app, version0.0.45-local.146, ASAR SHA256 a43df7c98b17e1a0465821fff103049e3a7ed33b221f153e2a413b8fe2a624a2. Binaire installé : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget, SHA256 eadffeeb9532bc9ddfbac7946e49c357a42de117cec9ab7f70fde481a902361d. Ancienne application et ancien binaire conservés dans /Users/moi/.cache/t3-install146.3URfCM. Aucun contenu de conversation ni base n'a été modifié par l'installation.

<a id="extrait-6"></a>

## Extrait 6

Installation sur disque, pas activation : PID et dates de démarrage inchangés avant/après pour T3 90726, son serveur90797, daemon68331 et pont68279. Aucun quit/open, lancement de seconde application ou commande LaunchAgent. Le panneau récent exige le prochain redémarrage de T3 et Bridget, qui reste non exécuté. Les compétences142, les autres worktrees et le fichier AppleDouble préexistant de T3 sont préservés. La dernière étape documentaire enregistre ces faits, sans modifier les sources testées.

<a id="extrait-7"></a>

## Extrait 7

Ce complément remplace le constat d'activation en attente ci-dessus, pas les preuves historiques d'installation. L'utilisateur a lui-même relancé T3 : application63000 et serveur63083, démarrés à13:53:41/42 CEST. Il autorise ensuite explicitement le redémarrage des deux services Bridget, sans relance T3. `launchctl kickstart -k` vise uniquement com.bridget.daemon puis com.bridget.t3. Nouveaux PID59975 et61796, démarrés à15:28:16 et15:28:28 CEST. Les deux services restent running ; les PID T3 restent inchangés. Les empreintes du binaire et de l'ASAR correspondent toujours à la livraison146.

<a id="extrait-8"></a>

## Extrait 8

L'utilisateur demande aussi vérification, suppression, livraison et cleanup, sans nouvelle PR T3. La base active /Users/moi/.t3/userdata/state.sqlite et la sauvegarde /Users/moi/.cache/t3-adoptions/spec143-20261007.KfjhE4/state-snapshot.sqlite passent SQLite quick_check en lecture seule. La sauvegarde complète de12 522 332 160 octets reste conservée. Les six autres copies SQLite, dont une tentative partielle, et son journal sont supprimés :68 931 617 792 octets logiques. Six anciennes copies d'application139/140/143 sont supprimées. Le retour arrière146 reste intact dans /Users/moi/.cache/t3-install146.3URfCM. Aucun contenu actif, secret de sauvegarde, conversation ou fichier temporaire macOS n'est supprimé.

<a id="extrait-9"></a>

## Extrait 9

Le nettoyeur existant est utilisé avec un budget ponctuel de zéro, après26 tests isolés PASS. Il retire les intermédiaires debug et le temporaire marqué145 :15,68 Gio logiques, clones APFS compris. Il conserve target/release utilisé, ses binaires et les verrous. Les cinq worktrees Bridget139/140/141/143/144 propres, fusionnés et inutilisés sont retirés. Six worktrees T3 locaux139/140/141/143/144/146 sont retirés après contrôle d'ascendance, de propreté et d'absence de fichiers ouverts. Cinq fichiers AppleDouble générés sont identifiés par leur signature, archivés puis retirés avant le contrôle de propreté. Aucune suppression forcée. Les branches restent conservées. Les travaux142 et145 modifiés, la branche portable139 non fusionnée et les autres projets restent intacts.

<a id="extrait-10"></a>

## Extrait 10

Gain physique mesuré sur le volume actif :79,3 Go environ. `df -k` passe de26 998 604 à104 474 280 Kio disponibles, soit environ107 Go libres après nettoyage. Les mesures évoluent avec les agents actifs ; elles ne sont pas la somme des tailles logiques. Les sauvegardes T3 passent de78 à12 Gio et target Bridget de9,9 à1,5 Gio. Scripts de suppression à listes fermées et métadonnées archivées : /Users/moi/.cache/bridget-cleanup146.HKwExt. Aucun nouveau build ni changement de source n'est nécessaire pour cette activation. Le complément documentaire est livré dans la même session146 ; aucune nouvelle PR T3.
