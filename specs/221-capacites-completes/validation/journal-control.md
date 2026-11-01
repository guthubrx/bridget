# Lecture du journal indépendante des actualisations de fil
Statut : correction en cours, session221/C27. Programme52/182 inchangé.

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
