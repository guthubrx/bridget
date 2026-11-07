# Décisions sur les bornes — 241

Ces valeurs viennent du code, pas d’une mesure de fréquence d’incidents. Elles
ne sont pas toutes des limites sur le travail : plusieurs protègent une trame,
une allocation, une attente ou la rétention d’une preuve. Aucun plafond de
160 demandes n’a été retrouvé dans ces parcours.

| Élément / source | Valeur antérieure | Décision et justification vérifiable |
|---|---:|---|
| storage_compatibility : nombre de contrats | 16 | **Supprimer**. Le rapport complet est déjà limité à 8 Kio ; identifiants bornés et uniques. Le nombre de contrats conservés ne mesure pas une ressource indépendante. Le candidat en déclare 15 : ne pas créer un refus artificiel à la prochaine évolution. Test avec 20 contrats supplémentaires accepté dans la même trame. |
| workspace_commit_view : volume sélectionné et audit | 32 Gio | **Supprimer**. Objets copiés et hachés en flux ; espace réel contrôlé par fstatvfs à la préparation. Garder les compteurs u64 sans débordement. Écrivain, lecteur, audit et matérialisation modifiés ensemble. Test manifeste 33 Gio accepté ; dépassement u64 refusé. Ce test ne prétend pas exporter 33 Gio. |
| workspace_git + admission : worktrees enregistrés et retenus | 128 | **Supprimer**. Ce nombre ne mesure aucune ressource occupée. Garder les claims uniques de création, contrôles de branche/base/chemin, erreurs Git/disque et capacité des admissions. fstatvfs compare avant création l’espace disponible au volume des fichiers Git à extraire (sélection existante pour compact, ls-tree sinon). Cette mesure ne réserve pas de place et ne prédit pas les allocations de métadonnées ni les écritures concurrentes : ENOSPC/EDQUOT restent classés pendant Git. Retirer aussi le scan de toutes les Task sous le mutex pour compter les anciens dossiers. Tests avec 128 vrais checkouts + réservations concurrentes. |
| workspace_commit_view : entrées, y compris chemins ancêtres | 65 536 | Conserver pour cette version : ensembles BTreeMap/BTreeSet et graphe de liens sont en mémoire ; profondeur 64. Cette limite porte sur les métadonnées de l’export, pas le volume des fichiers. Une conversion intégrale en inventaire externe serait un changement distinct, pas une simple hausse de constante. |
| workspace_git : sortie normale / inventaire | 1 Mio / 64 Mio | Conserver : buffers réels en mémoire, tronquage explicitement refusé plutôt que faux inventaire complet. Lecture des inventaires déjà portée à 60 s en 234. |
| workspace_compact : manifeste / commit_view intent / notice | 64 / 64 / 256 Mio | Conserver : preuves JSON lues intégralement, bornes écrivain et lecteur cohérentes depuis 234. Le volume des fichiers exportés n’entre pas dans ces budgets. |
| commit_view : liens / profondeur | 4 096 octets / 64 | Conserver : noms/liens et parcours bornés, refus des cycles et sorties de vue. Aucun lien long n’a causé les incidents examinés. |
| commit_view : export/audit | 30 min | Conserver : budget d’opération annulable ; ne pas renouveler le délai en boucle. Les erreurs ne se transforment pas en refus de droits. |
| remote_worker : store / entrées | 2 Gio / 4 096 | Conserver : reçus techniques en mémoire et sur disque ; réservations incluent les preuves et les orphelins. Pas un quota de données métier ni de RAM pour le programme enfant. La capacité se libère par clôture/rétention native, sans effacement de travaux. |
| remote_worker : trame / enveloppe résultat / corps résultat | 2 Mio / 2 Mio / 256 Kio | Conserver : framing JSONL et sortie résumée ; les fichiers produits restent dans le workspace. Ne pas charger un livrable entier dans le protocole. |
| protocole : sondes historiques / V2 | 8 / 128 candidats par lot | Conserver : compatibilité du protocole et mémoire bornée ; V2 sert les catalogues étendus. Un lot n’est pas une limite de missions cumulées. |
| origin_proof : preuves / préparations simultanées | 32 / 16 | Conserver : E/S lentes simultanées ; elles n’occupent pas le chemin de lecture d’état par clé. Refus de capacité distinct d’un changement de droits. |
| delegation : admissions actives globales / propriétaire | 128 / 16 | Conserver : réservations simultanées avant admission, pas total historique de missions. La même clé récupère le reçu existant et ne consomme pas une place nouvelle. |
| delegation : canon / métadonnées V1/V2 | 512 / 16 / 128 Kio | Conserver : stockage et parsing des reçus en mémoire ; V2 étend le catalogue sans accepter de contenu arbitraire. Les fichiers ne se transportent pas dans le canon. |
| delegation : brief de mission | 65 536 octets | Conserver : consigne au modèle, fichiers de référence dans le workspace. Aucun incident de ce chantier n’a établi une consigne légitime plus grande. Une limite dépassée doit nommer le champ ; ce n’est pas une absence de worker. |
| delegation_lineage : tâches retenues / rétention | 4 096 / 7 jours ; tombstones 30 jours | Conserver dans ce contrat : plusieurs parcours globaux utilisent encore un jeu en mémoire borné. Lecture et statut par clé doivent fonctionner à cette borne et sans lire l’historique. Ne pas purger des tâches non closes pour admettre une nouvelle demande. |
| mcp : appels simultanés / attentes | 8 / 4 | Conserver : threads et sockets par façade ; laisser de la place aux lectures/annulations pendant les attentes. Test d’annulation avant envoi ; aucun rejeu de mutation automatique. |
| client : budget ordinaire | 10 s | Conserver le budget unique, supprimer ses dépendances d’entretien étrangères. Une coupure précoce d’inscription reçoit une seule reprise dans ce budget ; refus d’identité non repris. Le délai historique reste à qualifier précisément, pas masqué par une hausse de timeout. |
| t3code : pages de détail | 320 | Conserver : pagination de l’adaptateur ; pas une autorité ni une limite de statut de Task. Les lectures ciblées ne dépendent pas du chargement de toutes les pages du client. |
| journal MCP : fenêtre | 200 messages | Conserver : fenêtre de lecture explicite ; ce n’est pas un quota d’envoi ou de stockage du dossier. Les reçus par clé servent à lire l’issue d’un envoi. |
| handoff : corps / résumé / objectif / éléments | 16 / 8 / 1 Kio ; 12 éléments, 16 références | Conserver : résumé structuré ; les sources et livrables restent référencés hors corps. Ne pas recopier un rapport entier dans chaque passation. |
| registry / prepare-project : mappings de projets | 64 | Supprimer le quota de projets. Le registre garde sa borne réelle de 16 Mio ; lecteur Python aligné sur le lecteur Rust (anciennement 1 Mio) et écrivain atomique contrôlé avant publication. Les profils et préférences gardent leur propre borne. |
| service_limits : descripteurs | 4 096 | Conserver : minimum logiciel de la limite souple, ne rabaisse jamais une valeur supérieure, ne change pas la limite dure du système. Ce n’est pas un plafond de tâches. |
| protocol : cache tardif 240 | 4 096 tuples | Conserver : mémoire technique ; aucune Task ni nouvelle mission. Preuves 240 réutilisées tant que ce code ne change pas. |

Une saturation réelle reste possible. Elle doit être mesurée et nommée. Une
absence de réponse demeure une issue inconnue. Aucun retour vers une version
qui refuse les nouveaux contrats ou volumes ne sera annoncé compatible sans
qualification du lecteur de retour.
