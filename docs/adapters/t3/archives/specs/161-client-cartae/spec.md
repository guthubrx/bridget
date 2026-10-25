# Archives techniques de l’adaptateur T3

Document d’origine : [specs/161-client-cartae/spec.md](../../../../../../specs/161-client-cartae/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Étant donné une préparation encore incomplète, quand une erreur survient, alors la commande courante reste inchangée.
2. Étant donné une préparation validée, quand la commande courante est remplacée, alors chaque nouvelle invocation trouve soit l'ancien client complet, soit le nouveau client complet.
3. Étant donné une communication non conforme après bascule, quand le retour arrière s'applique, alors la référence précédente est rétablie et le résultat est signalé comme non livré.
4. Étant donné les sessions actuelles, quand la livraison se termine, alors aucun redémarrage de T3, du maître, du pont ou de la liaison n'a eu lieu et aucun nouveau daemon Cartae n'existe.

<a id="extrait-2"></a>

## Extrait 2

- **FR-001** : installer un client issu exactement de la source 160 approuvée ; distinguer cette identité de celle de l'outil de livraison.
- **FR-002** : vérifier les prérequis et l'intégrité des sources avant la compilation ; refuser toute différence.
- **FR-003** : ne télécharger ni outil ni dépendance implicitement ; la copie explicitement tracée de dépendances déjà présentes sur le Mac et vérifiées contre le verrou approuvé est permise, sans changer ce verrou. Une préparation impossible laisse le client courant intact.
- **FR-004** : borner la compilation à deux travaux parallèles et à une priorité réduite ; une seule compilation du lot active à la fois.
- **FR-005** : préparer la nouvelle livraison dans un emplacement neuf et privé sans écraser une ressource existante.
- **FR-006** : vérifier l'identité, l'architecture et les droits du nouveau client avant son activation ; vérifier sa communication avec le maître existant.
- **FR-007** : remplacer la référence courante de façon atomique après validation et seulement si elle est toujours celle relevée au départ.
- **FR-008** : conserver et vérifier le retour arrière vers le client précédent ; restaurer cette référence si la recette après bascule échoue et si la référence courante nous appartient encore.
- **FR-009** : préserver sans redémarrage T3, le maître, son pont et la fédération ; ne pas démarrer de daemon Cartae ; aucune édition directe, migration ou suppression de base. Les inscriptions et messages synthétiques envoyés par la recette de communication sont autorisés par le périmètre validé et doivent être identifiés dans le reçu.
- **FR-010** : vérifier status et l'annuaire via le binaire candidat, puis un échange synthétique bidirectionnel via la recette transport existante, sans mission ni appel à un fournisseur de modèle. La recette transport ne prouve pas l'exécution du binaire.
- **FR-011** : livrer les changements du lot dans la branche cible et vérifier la référence distante après envoi.
- **FR-012** : produire un reçu durable distinguant Git, fichier installé et communication réellement observée ; ne pas présenter cette livraison comme une exécution distante de mission.
- **FR-013** : nettoyer uniquement les ressources inutiles appartenant au lot, après transfert et vérification des preuves ; justifier toute conservation.
- **FR-014** : conserver les refus de sécurité déjà présents dans l'outil de livraison et leurs vérifications, notamment les chemins privés et les hôtes connus.

<a id="extrait-3"></a>

## Extrait 3

Exécuteur distant, sélection de machine, choix de moteur, serveur de décision, changement d'interface T3, migration de maître, réplication de données, création de tunnel et redémarrage des services. Ces besoins restent dans les lots 162/163. Aucun test global Rust répété pour un changement limité au script de livraison et à ses tests.
