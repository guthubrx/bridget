# Validation et livraison 232 — 4 novembre 2026

Livraison exécutable activée : version0.8.1, build232-capacity-attestation-r2,
source5f4516edadf33c7c6b68441329b24a39abc48362 intégrée àmain et poussée.
Le complément documentaire ne change pas les octets compilés.

## Capacité et vérifications utiles

- Copies sélectionnées :512Mio→32Gio, soit64fois ; inventaire Git et manifeste64Mio.
- Réserve partagée du worker :256Mio→2Gio, pour conserver de la capacité avec des intentions de vue allant jusqu’à64Mio.
- Export et audit :30minutes. Délais métier, identités, empreintes, contrôle du disque et autres opérations Git conservés.
- Mesure du commit opus2D dcd7356f6ddd4d7a14e10165c4caea9f539e416a :32276blobs, inventaire4739540octets, sélection612886253octets. Cette sélection dépassait réellement512Mio.
- Export et audit réels de640Mio : source inchangée. Même cas refusé avec l’ancienne borne512Mio, puis réussi avec32Gio. La borne32Gio est également contrôlée sans écrire32Gio de données de test.
- 42tests de vues acquis,3ignorés préexistants ;1diagnostic fermé,1réponse perdue et3causes d’origine :47contrôles ciblés distincts. Le même test de diagnostic a été rejoué après le delta de revue, sans le recompter.
- Revue indépendante346c4f60-eb02-4a76-9664-5d406bbc0076, résultat lu et acquitté. DeuxP2 de guidage et uneP3 de validation traités. La récupération d’admission n’a pas été vérifiée par ce relecteur ; auto-relecture et test ciblé déclarés. Voir revue.md.

## Incident d’admission et message orphelin

La tâche702d272e-0652-4bc2-affb-5dd28817d8bd existait dès06:16:09UTC.
La lecture manuelle bridget_session a eu lieu à06:17:23UTC. Elle ne l’a pas créée
et n’a pas réparé des droits. Le premier appel a perdu la visibilité d’une
admission durable ; la même clé a ensuite retrouvé la même tâche.

Le maître a observé origin_proof_unavailable puis origin_worker_unavailable.
Le pont a fermé notre lien à06:17:16UTC ; les refus « agent_id déjà connecté »
ont persisté jusqu’à06:22:10UTC, avant une nouvelle exposition à06:22:55UTC.
La purge de présence et le signalement orphelin se placent dans cette coupure.
La cause initiale de la latence reste inconnue. Aucun effet d’une version Claude
ou du changement de fil n’est établi. Le correctif conserve les causes connues,
la clé et l’incertitude, sans nouvel appel de délégation ni gain de droits.

La clé fermée mcp-68988-6aeace2d-1 n’a pas été rejouée. Le diagnostic nouveau
mcp-15456-6aeadaf3-1 a été envoyé au propriétaire ; reçu in_flight conservé,
sans inventer une lecture. Dix-neuf préavis puis dix-huit avis de reprise et ce
diagnostic ont été envoyés sans réponse demandée. Ces avis ne créent pas de mission.

## Activation vérifiée

| Machine | Composants | PID après activation |
|---|---|---|
| Cartae | maître, mobile, worker |2926764,2926781,2928303|
| SYS | worker sysle2 |1954281|
| KS | worker ksstore |2739859|
| Mac | worker, pont natif |87396,87614|

Empreinte Mac :2923a1a78474b94fb75cd337b7110ea295d79af3a3b1ca260909f70b75732653.
Empreinte Linux :735d4b0fc5db6af41be805de491173313d90ac31cbfbeede3ed318a6831405e3.
Sept images réellement chargées contrôlées. Coupure du maître :2,27secondes.
Assemblage Linux sur Debian12/glibc2.36 ; symboles requis jusqu’àGLIBC2.34.

Les superviseurs Cartae2500061/birth270585460 et SYS1927195/birth94282125
ont gardé leur identité pendant le passage. Le registre Mac dédié est inchangé
(SHAacbc965ef4737c6a99c586334995b0797c750ea887c272dffd1217fd70fffcb2).
Les empreintes intégrales de7024 et77ba sont inchangées avant/après services.
Le canonique de702d reste176a4c85b86b5887c50c7f735105cd3f5f875eb10f5212e49758628280b830b2.
Son état peut évoluer par son travail autorisé. Aucun Start, Cancel, Fence,
rejeu, remplacement ou modification des services métier.

Les formats workspace.commit_view.large.v2 et worker.store.capacity.v3 sont
exigés. Le retour232-view-reader-r0 est vérifié dans les deux sens ; l’ancienR4
est refusé comme lecteur de retour. Les quatre pointeurs privés rollback ont
été écrits atomiquement sous verrou natif, avec sauvegarde et reprise de leur
seul outil de mise à jour. Aucun retour réel n’a été lancé.

## Client et publication

Fermeture/réouverture natives deT3 sans changement de son application231.
Empreinte app.asar conservée :81a55cefac78c12208e99a53f6f5008bcc484b571bd4e45809a81a1c5eef6ff0.
Les20anciennes façades MCP sont arrêtées ;18façades courantes232-r2 et leur
empreinte ont été contrôlées après reprise. Une lecture précoce primary_absent
pendant la republication n’a pas déclenché de mission ; le journal a ensuite
confirmé le retour du lien, puis who same_project a réussi, projet non nul,
agent bridge1016/conn-202. La preuve nativeV2/permissions du tour est présente.
Le déclencheur ponctuel de relance a été retiré après sortie0.

Releasepubliquev0.8.1 : sixactifs, tailles et empreintes confirmées parAPI.
APK8001 signé avec le certificat production
0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588,
la lignée existante et la configuration Firebase conservées. SHAAPK :
4d94ad030b79a20073c92b74c6fa28d54799d97acbd2db66bccd5315ba15ffbd.
Assemblage Android54s. Aucun changement de parcours graphique dans ce lot ;
les39tests antérieurs restent pertinents, sans nouvelle campagne ni recette
visuelle inventée. La réserve graphique de230 ne disparaît pas dans ce lot.

## Git, nettoyage et conservation

Branche232 locale et distante retirée après intégration vérifiée. Worktree de
compilation SYS propre, sans ignoré ni fichier ouvert : retiré sans force.
Onze copies de transfert ou binaires intermédiaires retirées. Sources temporaires
Android et produits identifiés :931fichiers retirés après comparaison aux
sources archivées et contrôle des usages. Total455151363octets logiques
retirés, hors fichiers du worktree. Le déclencheur LaunchAgent est aussi retiré.

ActifR2 et retourR0 conservés. Ancien checkout227 sale préexistant, caches dont
l’observation reste incomplète, anciennes images sous usage ou namespace non
qualifié, preuves/reçus/paquets publiés et vues de revue sont conservés.
La vue de revue232 reste sous conservation non confirmée : aucun marqueur effacé.
Le nettoyeur officiel Cargo a contrôlé avant/après et après installation ; surSYS,
le dernier installed refuse le nettoyage faute d’observation complète, sans forcer.

Preuves privées et reçus :
/Users/moi/.cache/bridget232-capacity-attestation/.
Les sources de correction et ce bilan sont poussés surmain ; l’alignement final
vérifie les racines principales des quatre machines sans déplacer les racines
historiques détachées deSYS etKS. Le dépôt clientT3 n’est pas modifié par232.
