# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/spec.md](../../../../../../specs/168-routage-flotte/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Santé finale reçue : B en ligne, catalogue 27, pont 32 fils vers 97328 ; DB readonly quick_check ok/FK0, versions 7/12 ; worker B 3767260 actif, configuration inchangée. Application T3 65544 et tunnel 60206 préservés ; le serveur 65608 a été remplacé extérieurement par 97328 né à 16:51:02 CEST, sans signal T3 de notre part. La restriction temporaire de 16:56 CEST est levée.

<a id="extrait-2"></a>

## Extrait 2

Limites : FULL unique 2820 PASS/32 FAIL/71 ignorés ; 29 échecs initiaux PASS après reprises, trois annulations 159 non validables sur Linux, cause initiale T3 inconnue malgré replay PASS. Deux LOW rustls-webpki et une INFO unsafe-libyaml restent documentés ; aucun GREEN global. Aucun nouvel audit/FULL/P4/build/service/test/modèle par cette rédaction. Aucun retour direct au binaire 0.3.0 après reçu admission v2. Les commits documentaires descendants conservent la source binaire B et ne nécessitent aucune reconstruction.

<a id="extrait-3"></a>

## Extrait 3

FULL : 2 820 PASS / 32 FAIL / 71 ignorés. Après reprises ciblées, 29 des 32 échecs initiaux passent ; trois annulations 159 restent non validables sur Linux (observateur kqueue Darwin/BSD). La cause du premier échec d’identité T3 reste inconnue malgré son replay PASS. Aucun succès global vert n’est déduit.

<a id="extrait-4"></a>

## Extrait 4

L’utilisateur veut confier des sous-tâches à sa flotte depuis Bridget, sans dépendre de T3.
Le choix doit tenir compte des capacités réellement disponibles, des ressources des machines
et des limites attestées des comptes. Le modèle, sa connexion et sa machine sont trois notions distinctes.
Une machine saturée peut perdre la préférence donnée à son dépôt ou à son cache déjà préparé.

<a id="extrait-5"></a>

## Extrait 5

1. Le commit de départ exact doit être disponible sur la cible avant tout lancement de code.
2. La continuité d’un dépôt ne force pas la machine ; le coordinateur peut préparer la même base ailleurs.
3. Le résultat indique la base, la branche, la tête et les modifications non enregistrées.
4. Un résultat disponible ne signifie pas que le travail est intégré.
5. Un intégrateur autorisé récupère et vérifie le résultat avant nettoyage ; le daemon ne fait
   aucun fetch, pull, push ou merge de livraison.
6. T3 peut rester fermé pendant tout le parcours.

<a id="extrait-6"></a>

## Extrait 6

- **SC-001** : Dans un namespace isolé sans serveur, pont ni configuration T3, une mission automatique locale
  et une mission distante terminent avec résultat corrélé, sans arrêter le T3 de l’utilisateur.
- **SC-002** : Un catalogue de douze combinaisons au moins reste complet et sélectionnable sans troncature à huit.
- **SC-003** : Dans chaque scénario de contrainte explicite testé, aucune sélection ne viole un axe imposé.
- **SC-004** : Deux comptes distincts du même fournisseur conservent des faits de quotas séparés dans tous les scénarios prévus.
- **SC-005** : Dans la chaîne réelle, une machine franchement saturée ne peut pas être choisie par le service
  ou le petit décideur lorsqu’une autre machine admissible est prouvée disponible, malgré le bonus de continuité.
  Une machine imposée saturée refuse sans bascule ; le rejeu conserve le choix déjà durable.
- **SC-006** : Les trois issues service valide, petit décideur valide et dernier repli sont chacune prouvées.
  Le service choisit sur un ensemble entier qui tient dans ses bornes attestées, huit options au plus par
  prudence initiale ; la matrice12+est prouvée par le petit décideur réel et le score, avec indisponibilité
  de capacité explicite si le service ne peut tout représenter. Les niveaux désactivés émettent zéro consultation.
- **SC-007** : Toute chaîne rend un choix ou une issue explicite avant le budget configuré,
  hors préparation Git dont le délai reste séparé et borné.
- **SC-008** : Un rejeu et deux demandes concurrentes donnent au plus une admission ;
  après redémarrage, zéro nouvelle consultation de décision pour cette clé.
- **SC-009** : Aucun secret de configuration ou d’authentification, chemin d’authentification ou outil
  métier/système/réseau/MCP n’est automatiquement injecté. Seul StructuredOutput formate le JSON
  sans droit d’exécution ; les autres outils/tool_use sont rejetés et la sortie est validée localement. Le brief utilisateur est borné et inerte,
  sans garantie de détection universelle de secrets ; une réponse invalide ne déclenche aucun effet.
- **SC-010** : Chaque lancement de code prouvé part du commit demandé ; chaque nettoyage final
  dispose d’une preuve de récupération et d’absence de travail vivant ou étranger.
- **SC-011** : Les noms réels et capacités de chaque modèle/connexion activés sont vérifiés sur leur machine ;
  la documentation distingue clairement les possibilités déclarées et l’activation observée.
