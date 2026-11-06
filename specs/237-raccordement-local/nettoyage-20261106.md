# Nettoyage demandé le 6 novembre 2026

François demande la consolidation dans main et le retrait maximal des branches,
worktrees, copies exécutables et caches obsolètes. T3 doit rester ouvert : son
rechargement manuel est reporté par l'utilisateur. Aucun service n'est relancé.

## Git et conservation des sources

Les sources historiques du lecteur de retour 234 et des branches worker/retour
198 sont rattachées à main. Trois copies modifiées sont conservées sous forme
de commits historiques, puis rattachées à main :

- Cartae 227 : e9ef9bb7c5162704e0c1cd61188d598beca68e51, 28 fichiers ;
- KS Android 222 : b7c370bcc02d5bbc1ef0653cac8bc8c23d2c39a2, 8 fichiers ;
- SYS 227 : 826b40bc50746fd79266e1995a772ce21eb73f0c, 63 fichiers.

Ces rattachements conservent les anciennes sources dans l'historique. Ils ne
réintroduisent pas leurs anciens réglages ni leurs versions intermédiaires dans
le code actuel. Les branches 198 examinées ont leurs changements déjà intégrés ;
le lecteur 234 est une variante historique de retour. Les copies 227 et Android
222 sont conservées comme états historiques, sans nouvelle qualification métier.
Les fusions emploient explicitement la stratégie ours. Avant et après ces
rattachements, l'arbre reste d04833290cfd70bbf1071cee63aca33ed837cfc1.
La consolidation 0e21d201c03ee7bcbba21dc89c973c177745d059 est poussée.

Les copies exportées sont contrôlées par SHA-256. Le contrôle de secrets ne
trouve rien dans les copies Cartae/KS. L'unique alerte SYS porte sur la chaîne
de test idempotency_key « guidance156-cli », sans secret réel. Les anciens
commits sont conservés comme parents ; aucun test ni build applicatif n'est
répété puisque le code actuel est inchangé.

## Retraits vérifiés

| Cible | Worktrees supplémentaires retirés | Branches locales restantes |
|---|---:|---|
| Mac | 1 | main uniquement |
| Cartae | 6 | main et bridget198-worker-start |
| KS | 17 | main uniquement |
| SYS | 0 | 92 branches, 98 worktrees supplémentaires conservés |

Le distant GitHub n'a plus que main. Les miroirs techniques Bridget SYS/KS
n'ont plus que main également : 11 références historiques déjà intégrées sont
retirées par comparaison d'ancêtre et mise à jour atomique avec SHA attendu.

Les contrôles privilégiés des processus sont complets pour les retraits Cartae
et KS. Les tâches correspondantes sont closes, sauf le worktree explicitement
conservé ci-dessous. Les fichiers ignorés régénérables sont retirés avec leurs
worktrees. Les preuves ignorées sont archivées hors du dépôt avant retrait :
232 fichiers d'audit et 4 fichiers de configuration/mémoire. Les archives sont
également recopiées sur le Mac. Les reçus et les enregistrements des tâches ne
sont pas modifiés par ce nettoyage.

Les caches Cargo sont entretenus avec le script natif et un budget temporaire
nul pour ce nettoyage explicitement demandé : environ 2,72 Gio sur le Mac et
8,26 Gio sur Cartae. S'ajoutent les worktrees retirés, les anciennes images
installées et les exécutables de test/préparation devenus inutiles. Le total
est d'environ 23 Gio de fichiers et caches retirés, en tailles logiques ; les
clones APFS peuvent réduire le gain physique réel sur le Mac. Les répertoires
out, build et dist n'existent pas à la racine Mac.

Le rangement Git standard conserve les références et les sauvegardes : aucun
prune immédiat des objets inaccessibles n'est utilisé. Le stockage Git Mac
passe d'environ 140 Mio d'objets à 45 Mio, hors petit coût des métadonnées.
Sur KS, l'espace disponible mesuré passe d'environ 0,15 Gio avant cette
campagne à 4,09 Gio au dernier contrôle. Ce dernier chiffre inclut l'activité
éventuelle des autres travaux sur le même système de fichiers.

## Conservations nécessaires

- Cartae : /home/moi/10.Scripts/64.bridget/.worktrees/1cc44f4d reste utilisé
  par le processus 2500061. Sa tâche 1cc44f4d-f064-4375-a922-1bdf87a05400
  n'a pas fini son nettoyage natif. Son fichier modifié, sa branche et l'image
  183 encore chargée sont conservés. Aucun processus d'un autre travail n'est
  arrêté pour faire le ménage.
- SYS : la lecture complète des processus exige sudo. L'authentification n'est
  pas fournie ; deux cartes sécurisées expirent sans réponse. Les 98 worktrees,
  leurs branches et environ 16 Gio de caches de compilation restent conservés.
  La copie source modifiée 227 est déjà sauvegardée dans l'historique de main.
  Le checkout principal reste sur son ancien HEAD détaché ; la référence main
  est mise à jour. Le prochain passage doit qualifier les usages avant retrait
  et avant changement du checkout principal.
- Cartae : 19 exécutables de préparation en cache, environ 0,74 Gio, restent
  conservés. Le lecteur natif des processus ne fournit pas une observation
  complète pour cet ensemble lors des essais de retrait. Aucun effacement
  forcé n'est effectué ; ce refus local ne bloque pas les autres nettoyages.
- Mac : quatre anciennes réserves stash restent conservées avec leurs
  références. Leur contenu n'est pas réappliqué au code actuel. Les archives,
  reçus, sources non qualifiées et preuves restent hors du nettoyage aveugle.
- Les scripts de publication privés et le retour de l'application cliente sont
  conservés. Le nettoyage concerne Bridget, pas les dossiers out des projets
  métier ni les travaux d'autres agents.

## Vérification finale

Les sept composants gardent leurs PID et images 237-local-project-r2 : Mac
57995/58232, Cartae 1605276/1605395/1608385, SYS 2354435, KS 3439620. Les SHA
exécutables et les pointeurs de retour 236-origin-local-r2 sont relus. Le
registre spécial Mac garde son empreinte attendue. Les payloads 7024, 77ba et
702d ont les mêmes SHA qu'avant ce nettoyage. Aucune mission n'est créée,
rejouée ou annulée. Aucune revue n'est recommencée.

La vérification des façades clientes reste reportée jusqu'à la fermeture et
réouverture manuelles de T3 décidées par François. Ce nettoyage ne clôture pas
cette étape et ne déclenche aucune relance.

Les inventaires, archives de preuves, SHA des fichiers retirés et comptes rendus
détaillés sont conservés dans :
/Users/moi/.cache/bridget-cleanup-20261106

Sur les serveurs, les comptes rendus locaux sont conservés dans :
/home/moi/.cache/bridget-cleanup-20261106

Le résultat demandé « aucun worktree ni branche supplémentaire partout » reste
incomplet pour les deux raisons concrètes ci-dessus : usage actif Cartae et
authentification SYS manquante. Aucun nettoyage n'est annoncé pour ces éléments.
