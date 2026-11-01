# Lecture du journal indépendante des actualisations de fil
Statut : livré, chargé et vérifié en fonctionnement. Session221/C27, programme52/182 inchangé.

## Incident et données réelles
Signalementmonuments opus201-incident-journal-jsonl-20261101-01 :
bridget_journal(agentaf5ff0db...,tail8) échoue à03:52UTC.
Une lecture identique est reproduite avant correction en11786ms :
« réponse daemon indisponible : budget JSONL dépassé ».
read_unix_line émet ce texte sur ErrorKind::TimedOut. Ce n'est pas une
preuve d'une réponse trop grande. Les bornes restent200entrées/64Kio,
avec le budget total de lecture10s et les limites de trame existantes.

Échantillon du pontMac9806 : worker du filmonuments32e1b8f8 bloqué dans
Client::snapshot ou sur le mutex deSession. Le lecteur réseau est distinct,
mais rangeait Subscribe/Unsubscribe dans la file de ce worker. Son tick
peut attendre HTTP, le mutex partagé et la projection de l'historique.
Le relais Attach a déjà son thread et une file bornée ; son accès attendait
donc un traitement qui n'était pas nécessaire à une lecture de journal.

## Correction
Port cloneable du même relais, mêmes commandes, état borné et annulations.
Le lecteur réseau de ce seul lien transmet Subscribe/Unsubscribe directement
au relais. Un contrôle consommé ainsi n'entre pas aussi dans la file du worker.
Les remises, annulations de mission et autres trames gardent leurs chemins.
Aucun nouveau protocole, lecteur disque client, fournisseur ou droit.
Les refus de saturation sont rendus par le même Writer authentifié.

## Deux demandes signalées en parallèle
Lecture du registre natif, sans rejeu :
- opus201-patrimoine-nouveau8-20261101-01 admise sous
  17c18e3c-541e-4886-804e-263eb6b3309a, result_available, sans erreur.
- opus2d-r55-borne-producteur-vues-revue-20261101-10 :
  refus durablepermission_not_inherited, tasknull, misàjour1793506117.
  Une bridget_session réussie seule ne prouve pas l'héritage de cette admission.
Aucune mission relancée, aucune enveloppe changée.
Le rendu isolé tenant un verrou métier signalé par3315 reste distinct :
aucun défautBridget n'est établi par ce seul fait.

## Revue et complément
Revue indépendante db7d13ed en lecture seule, résultat lu et acquitté ;
clôture propriétaire demandée une fois. Le mécanisme est cohérent, sans P1
confirmé. L'observation de lint sur les huit paramètres du lecteur est traitée
par l'annotation déjà utilisée dans ce module. Documentation du relais replacée.

Deux corrections suivent la revue, avec contrôle ciblé :
- le lecteur réseau ne prend plus le mutex du writer, même pour un refus de
  saturation. Il remet le refus fermé dans la file existante du worker.
- une erreur disque retirait la souscription mais gardait son identifiant dans
  active. La fin sur erreur libère désormais active et cancelled avant End ;
  même ordre sur remplacement/troncature de source.
Le test d'erreur disque remplace réellement le fichier par un dossier, reçoit
End, désabonne dans le callback puis réabonne le même identifiant. Il vérifie
absence d'entrée retenue et seconde lecture complète. Le test de saturation
retient le mutex du writer et sature le relais : le lecteur remet le refus
puis Disconnect sans attendre ce mutex. Le delta est auto-audité ; aucune
seconde revue générale n'est revendiquée.

Vingt-cinq contrôles Linux distincts acquis : nouveau test nominal, huit
régressions relais et treize du lecteur, deux nouvelles contre-épreuves
(saturation et erreur disque), plus remplacement/troncature de source.
Les trois tests nouveaux sont repassés ensemble après le complément final ;
les répétitions ne sont pas comptées comme de nouveaux tests.
La première compilation du nouveau test signalait un import encode manquant ;
qualification complète du nom corrigée avant les PASS.

## Vérification réelle et second point d'attente
Après chargement r2, la même lecture expirait encore en12248ms puis13350ms
pendant un échantillon de pile. Aucune réussite n'a été annoncée.
Le relais reçoit bien le contrôle ; il est alors dans resolve_window(Tail),
last_sequence_in_files puis valid_events. Le journal réel de monuments
couvre14 fichiers, environ53Mio ; le dernier fait environ4,7Mio.
refresh_files résolvait Tail à chaque tranche128Kio alors qu'il utilisait
seulement les fichiers de cette résolution. Le seuil from_seq restait figé.

Correction complémentaire : réutiliser la liste déjà résolue par Seq(0)
pour Tail durant le rejeu. Les listes de fichiers sont identiques ; aucune
séquence, fenêtre initiale, lacune ou borne n'est changée. Le travail répété
quadratique devient une résolution initiale et une lecture incrémentale.
Le contrôle direct du reader reste utile : il élimine séparément l'attente
de traitement HTTP avant de démarrer cette lecture.

La contre-épreuve réelle avec from_seq38372, sur la même source r2,
rend22événements, complet, sans notice en3079ms. La source et le transport
fonctionnent ; cette fenêtre n'exécute pas le recalcul Tail répété.
Le test socket nominal est revérifié PASS après le complément de cache.
Aucun nouvel essai métier ou changement de borne ; le delta de performance
réutilise exactement la même liste de fichiers.

## Livraison finale r3
Sources a206ec33, 7c4e921d puis 1249ea50859be09036ae9d8569505b1bf0409a7e
intégrées et poussées. Les assemblages reprennent les mêmes356 fichiers que
main ; SHA des sources vérifié sur Mac et SYS. Build221-journal-control-r3 :
- Linux : SHA2568381c9f1795e3db503555fc5bc4649a123cb54d8d3e6984aa6155917585b7971,
  41363648octets.
- Mac : SHA256d73aaeb9f1b652a586b0cc8e282f0faf855059a9c78402be76c43be4d3f29dfd,
  34180992octets.
Images réellement chargées et hachées :
Cartae maître120326, mobile120393, worker124411 ; SYS1192936 ;
KS1582629 ; Mac worker7096 et pont7218.
Deux passages maître : r2 en2,41s puis r3 en2,21s. WrapperCartae2500061,
naissance270585460, conservé lors des deux passages. Aucun service métier,
aucune mission historique ni aucun processus du rendu n'a été relancé.

Après r3 sur le pont : le même appel tail8 rend8entrées complètes en2648ms,
sans notice, séquences38433..38440, through38440.
Après chargement des sept services : le même appel rend8entrées complètes
en7610ms, sans notice, séquences38444..38451, through38451, next38452.
Ces temps sont deux observations, pas un engagement de latence.
Capabilities revient :45options déclarées, gel=false. Aucun nouveau modèle
appelé pour qualifier. Une lecture du journal ne prouve aucun succès métier.

Le retour natif r2 conserve les mêmes formats et marqueurs ; compatibilité
avant/arrière, image et SHA vérifiés. Il contient le correctif de contrôle,
mais pas l'optimisation finale Tail : ne pas le qualifier pour ce cas tail8.
Les pointeurs rollback Cartae/Mac sont écrits atomiquement sous verrou puis
relus. La version r1, seulement préparée, n'est jamais déclarée activée.
Le clientMCP ouvert reste ancien ; la correction s'exécute dans le pont/relais,
le schéma de lecture ne change pas et l'appel natif courant est vérifié.

124090944octets de copies de transfert hachées retirés surMac/Cartae/KS.
installedMac/SYS refuse encore le nettoyage pour observation incomplète des
processus ; aucun forçage. Actif, retour, preuves et images/worktrees sous
usage ou observation incomplète conservés. Pas de diaporama : correction
technique vérifiée par sockets et appel MCP réel.
Les demandeurs sont informés par messages Bridget, sans rejeu métier.
