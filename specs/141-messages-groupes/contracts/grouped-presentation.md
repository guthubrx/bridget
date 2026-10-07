# Contrat141 — Lot direct et présentation

## Entrée existante

Producteur : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:3033.

Le producteur concatène exactement :

```text
📥 {total} messages Bridget groupés dans ce tour (reply=no) :\n
\n── {index}/{total} — de {sender_label} (id {message_id}) ──\n{body}\n
```

La seconde ligne de format est répétée pour chaque message. Les indices vont de 1 à total. Le producteur ajoute ensuite un retour de ligne et le suffixe suivant :

```text
Pas d'accusé de réception pour ces messages : n'envoie aucun « bien reçu ». Ce n'est pas une absence de tâche : si un message demande une action, fais-la. Le pont ne relaie pas ta réponse finale pour ce tour ; pour transmettre un résultat à un expéditeur, fais un nouvel envoi Bridget.
```

Les écritures \n ci-dessus représentent les retours de ligne du format. Elles ne sont pas des caractères littéraux du message.

## Projection

Précondition : texte source disponible ; aucune confiance particulière dans son contenu. Effets de bord : aucun. La projection conserve le texte source.

La reconnaissance de l'en-tête reste ancrée au début et bornée à 1024 caractères. Le découpage vérifie le total, chaque indice, chaque dénominateur et les limites. Les lignes ressemblant à un séparateur ne sont jamais silencieusement consommées comme du texte ou des métadonnées si elles rendent les limites ambiguës. Un nom absent, un total invalide, un indice répété, une limite manquante ou un en-tête trop long impose le repli intégral.

Une section valide conserve le corps par tranche de texte. Seuls les retours de ligne ajoutés par la concaténation du producteur sont retirés. Les retours de ligne appartenant au corps sont conservés. Un suffixe exactement connu et final peut être masqué. Tout suffixe inconnu, modifié ou situé dans le corps reste visible.

La collection projetée est facultative. Sans collection certaine, le corps complet demeure lisible. Un en-tête non reconnu garde le rendu historique intégral. Aucun cas de repli n'ajoute un panneau source.

## Sortie utilisateur

Le groupe fermé occupe une ligne. Le groupe ouvert montre les sections dans l'ordre reçu. À sa première ouverture, seule la première section est ouverte. Chaque section a un nom fourni et un aperçu littéral limité aux 120 premiers caractères du corps. Chaque section peut ensuite être ouverte indépendamment. Refermer le groupe conserve ces choix pour le même fil/message. Son corps utilise le rendu Markdown existant, sans exécuter le HTML reçu.

Quand un nom humain est fourni, les UUID ou références de sous-agent reconnus peuvent être retirés de ce libellé. Si l'expéditeur est seulement un UUID, cet UUID reste affiché à l'identique. Il n'est pas remplacé par « Bridget ». Les autres éléments du nom sont préservés.

Le lot direct ne propose ni Sources, ni Détails techniques, ni vue brute secondaire. Cette règle vaut aussi si son découpage échoue. Le bouton de copie existant reçoit row.message.text original pour les seuls lots directs, même en repli sûr. Sa copie conserve notamment [x](t3-context://v1/skill/ctx_1) sans contexte structuré ; elle ne transforme pas ce lien en x. Les autres familles gardent leur copie actuelle. Le helper partagé n'est pas modifié. Pièces jointes et actions ne sont pas déplacées.

## Accessibilité et portée

Les commandes sont des boutons natifs avec nom accessible, aria-expanded, aria-controls et focus visible. Les clics et les activations clavier utilisent la même transition. Chaque changement de hauteur informe ctx.onToggleWorkEntry avec l'ancrage existant. La clé fil/message existante isole l'état.

Ce contrat ne s'applique pas aux lots 🔔, messages directs unitaires, sollicitations ou messages ordinaires. Il n'atteste pas une origine. Il n'introduit aucun contrat réseau, stockage, endpoint ou migration.
