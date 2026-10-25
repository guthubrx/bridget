# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/implementation.md](../../../../../../specs/140-bulles-compactes/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Périmètre autorisé : message.rs, daemon.rs, t3code.rs et tests proches côté
Bridget ; MessagesTimeline.logic.ts/.logic.test.ts et MessagesTimeline.tsx/
.test.tsx et logo côté T3. Recherches et décisions dans reuse-audit.md.
Autres branches et modifications préexistantes préservées.

<a id="extrait-2"></a>

## Extrait 2

Recette isolée : état synthétique créé dans
/Users/moi/.cache/t3-spec140-preview.9yd2HW, aucun accès en écriture à
/Users/moi/.t3/userdata. Disque externe sans hardlinks : home temporaire déplacé
sur disque interne après ENOTSUP à la création environment-id. Serveur arrêté
avant fixtures SQL ; helper t3-sqlite-state.ts a sauvegardé chaque mutation.
Deux fils synthétiques et cinq familles d'enveloppes, aucun agent lancé.
Modèle JSON ajouté aux fixtures après erreur de décodage constatée.

<a id="extrait-3"></a>

## Extrait 3

Projection pure des cinq familles dans la logique T3 existante. Branche Bridget du composant de repli existant, bouton natif et logo officiel importé localement. Le texte source, sa copie et ses détails bruts restent identiques. Les suffixes finaux ne sont retirés du corps lisible que par correspondance exacte. Aucun destinataire ou type de publication n'est inventé.

<a id="extrait-4"></a>

## Extrait 4

Sources contrôlées : quatre Rust (message.rs, communication.rs, daemon.rs, t3code.rs), deux sources frontend production et leurs deux fichiers de test, un SVG identique au logo officiel. Le diff est entièrement SPEC140. L'untracked étranger native/resource-monitor/._target dans la racine T3 est préservé. Aucun scan intégral des dépôts revendiqué.

<a id="extrait-5"></a>

## Extrait 5

- Baseline T3 :211 tests PASS. Suite ciblée finale :269 PASS,194 logique et75 UI,7,13s. Aucune mesure instrumentée de couverture de tests.
- Correction déléguée :12 tests RED avant correction puis GREEN ; six variantes inconnues préservées ; un test UI de copie brute exacte.
- Rust :67 tests distincts PASS :45 core, six SPEC140 daemon,11 communication, un SPEC102, quatre SPEC114. BRIDGET_HOME=/tmp/b140.uk4wWV, CARGO_TARGET_DIR partagé avec le target racine ; aucune connexion fournisseur/daemon actif.
- Lint ciblé :exit0 en2,33s.22 warnings, identiques à la baseline comparée en3,06s. Aucun warning nouveau confirmé.
- TypeScript web :exit0 en18,42s. Build web :exit0 en43,71s ; avertissement non bloquant des chunks>500ko.
- Format web :exit0 en1,19s. Rustfmt quatre fichiers et git diff --check PASS. cmp du logo :identique.
- cargo check --workspace --all-targets --offline :exit0 en19,17s, trois crates et leurs bins/tests ; aucun téléchargement fournisseur.

<a id="extrait-6"></a>

## Extrait 6

Commande de suite frontend ciblée : `vp test run src/components/chat/MessagesTimeline.logic.test.ts src/components/chat/MessagesTimeline.test.tsx`, depuis /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact/apps/web. Le build utilise `vp build` et les types `tsc --noEmit` dans ce même scope. Les contrôles Rust utilisent cargo test avec les filtres des suites nommées ci-dessus. Les commandes shell complètes de ces exécutions restent dans les traces du principal ; aucun run global n'est substitué à ces résultats ciblés.

<a id="extrait-7"></a>

## Extrait 7

Browser T3 seul, composant réel et état synthétique dans /Users/moi/.cache/t3-spec140-preview.9yd2HW. Enter et Space ouvrent la carte et conservent le focus. Les détails complets sont accessibles. Titre réel « Coordination politique », ancien format « Fil partagé ». Les deux thèmes ont été contrôlés.

<a id="extrait-8"></a>

## Extrait 8

Le resize natif de l'aperçu a été ignoré puis l'outil a expiré après10s. La largeur320 a donc été contrôlée dans une iframe de même origine via Browser T3, sans autre moteur de navigateur. Première recette sombre :six cartes44px et ancrage177 →177 →177. Recette finale claire après données fraîches :12 cartes44px dans320×800 ; débordement de toutes les cartes et du document0.

<a id="extrait-9"></a>

## Extrait 9

Le brut7ko DOM a finalement été récupéré après réouverture de Browser T3 :7357 caractères exacts comparés au GET source. Les deux premières erreurs de clic étaient côté client Browser. La capture PNG finale snapshot/save a échoué deux fois côté outil :aucun screenshot final validé, malgré les mesures DOM établies.10ko reste automatique React, pas un benchmark de charge. Ces erreurs d'outil ne sont pas des findings applicatifs. Serveur arrêté et fixtures sauvegardées avant chaque mutation. Aucune donnée active de /Users/moi/.t3/userdata écrite.

<a id="extrait-10"></a>

## Extrait 10

JSCPD Rust :quatre fichiers de contexte,35416 lignes,41 clones,734 lignes dupliquées,2,0725%. Intersection réelle des intervalles clones avec les lignes git diff -U0 :zéro. Web :deux sources production,6936 lignes,zéro clone détecté. Ces nombres sont du contexte, pas un taux du diff. Aucune baseline : newClones=0 ou isNew=false ne serait pas une preuve ; l'intersection réelle a été utilisée. Rapports dans /Users/moi/.cache/t3-spec140-preview.9yd2HW/jscpd-rust-full/jscpd-report.json et /Users/moi/.cache/t3-spec140-preview.9yd2HW/jscpd-web/jscpd-report.json.

<a id="extrait-11"></a>

## Extrait 11

Prochaine action recommandée lors de la clôture de développement :livraison explicitement autorisée des deux dépôts Bridget/T3, avec sauvegarde, compilation du paquet desktop et vérification après installation. Le développement seul n'a exécuté aucune de ces actions.

<a id="extrait-12"></a>

## Extrait 12

L'utilisateur autorise maintenant commit, fusion, push et installation de SPEC140 dans Bridget et T3. Cette section consigne la préparation, pas une livraison déjà accomplie. Le principal prend en charge les opérations Git, les builds et les services. À cet instant documentaire, aucun déploiement n'est revendiqué.

<a id="extrait-13"></a>

## Extrait 13

Les faits de livraison vérifiés et le plan d'activation T3 sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/livraison.md. Bridget est livré et activé ; les sources T3 sont fusionnées/poussées et le paquet signé est prêt. L'installation T3 reste à confirmer par le job et son reçu réel. Les sections antérieures décrivent les états historiques au moment de la validation et de la préparation, pas l'état de livraison courant.
