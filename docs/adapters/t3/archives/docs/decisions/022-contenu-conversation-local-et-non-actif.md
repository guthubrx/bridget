# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/022-contenu-conversation-local-et-non-actif.md](../../../../../decisions/022-contenu-conversation-local-et-non-actif.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Le journal Bridget reste l'autorité des tours. Un adaptateur de projection
   pur construit des tours stables sans changer la remise ni inventer un nouvel
   protocole agent.
2. Les réponses agent sont des documents à gauche, les demandes humaines des
   bulles concises à droite. Les actes, erreurs, durées et raisonnements
   attestés deviennent un résumé repliable rattaché à leur tour.
3. Le Markdown reste analysé par `marked` puis assaini par DOMPurify. Les
   éléments enrichis sont ajoutés après assainissement avec une liste blanche
   étroite. Aucun HTML agent ne devient exécutable.
4. Les préférences `liens externes`, `références de fichiers` et `images`
   sont séparées, versionnées, désactivées par défaut et locales au client.
   Dans Bridget Desktop, le store Tauri est l'autorité : le WebView de relais
   ne reçoit qu'un instantané en lecture injecté au démarrage. Il ne possède
   aucune commande Tauri d'écriture de ces réglages.
5. Les liens externes exigent un geste explicite. Dans Bridget Desktop, le
   lien est transmis à un schéma local `bridget-open:` que la coque intercepte
   puis revalide avant de demander l'ouverture HTTPS au navigateur système ;
   le WebView distant ne reçoit aucune capability Tauri. Les références de
   fichiers ne donnent accès qu'à un aperçu relayé, en lecture seule, après
   clic et seulement si le chemin canonique appartient aux racines de projet
   déjà autorisées. Les images distantes n'emploient que HTTPS,
   `referrerpolicy` `no-referrer`, chargement différé et une taille bornée. Les
   formats actifs et les URI `data:`, `javascript:` et `file:` ne sont jamais
   rendus.
6. Les extraits T3 repris textuellement ou adaptés gardent un commentaire de
   provenance et l'avis MIT de T3 Tools Inc. est ajouté aux notices tierces.
   Le reste reprend ses principes, pas son architecture React.
