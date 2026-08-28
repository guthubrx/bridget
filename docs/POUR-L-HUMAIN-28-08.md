# Pour l'humain — état au 28/08 17h42

> **Pourquoi ce fichier existe.** La route `v1/journal?agent=humain` rend **0 octet** alors que
> `agent=jc2` rend 28672 et `agent=jc1` 20480 — mesuré à 17h41 avec le jeton UI. Tu ne peux donc pas
> lire ce que je t'écris, et tu me le fais lire par un tiers. Tu viens de me redemander une chose à
> laquelle j'ai répondu **cinq fois** entre 17h11 et 17h24. Ce fichier contourne le canal cassé :
> tu peux l'ouvrir directement.

## Tes deux écarts — traités, et l'un n'est pas résolu

**ÉCART UN — CONFIRMÉ, NON RÉSOLU.** Ta mesure est reproduite : `agent=humain` rend 0 octet.
Cause trouvée : le correctif existe dans le commit `16be24f`, sur la branche `deploiement-courant`.
**Il n'est PAS dans `main` — huit commits hors main — et le binaire en service date du 28/08 11:33,
antérieur.** J'avais annoncé ce défaut « corrigé et vérifié en réel, 83 bulles ». C'était vrai à
l'instant de ma mesure et faux pour la durée.
**Ce qui débloque : l'intégration puis le déploiement. Je ne remplace pas le binaire sans ton accord** —
je l'ai fait ce matin pendant que le daemon tournait dessus et j'ai cassé le spawn pendant deux heures.

**ÉCART DEUX — MESURÉ, ET PROBABLEMENT PAS CE QUE TU CROIS.** Trois messages portent `sender=humain`
depuis 12h00 : le tien de 14:52:20, puis `T2 sonde attribution nom enregistre` (15:41:17) et
`test B nom enregistre` (16:37:24), **tous deux adressés à rc7-flux**, qui porte le mandat sur
l'identité d'émetteur. **Ce sont des sondes de test, pas des usurpations.** Si c'est toi qui les as
envoyées pour l'éprouver, l'écart est à zéro.
Ma mesure du 27/08 rend **cinq** messages `sender=humain` sur la journée, pas 428 — ton chiffre
portait sur autre chose. Dis-moi ce que tu as compté et je referai la mesure sur le bon objet.

## Ta règle est adoptée, et je l'ai violée dans les deux minutes

*Une clôture sur ancestralité prouve qu'un code est dans main, pas qu'il produit son effet.*
J'ai répondu en citant une mesure **avant qu'elle s'affiche**. Corrigé dans la minute.

**Découverte incidente** : le champ `reference` du registre impose le format `mesure:<texte>`.
**Le schéma exigeait déjà ta règle** ; je ne l'avais jamais rencontrée faute d'avoir passé la validation —
le catalogue était refusé par son propre parseur depuis 12h04, quatorze défauts en cascade. Réparé.

## Ce qui t'attend, et rien n'avance sans toi

1. **Deux arbitrages** demandés par jc2-flux sur ta priorité du 02h40, en attente depuis 16h31 :
   un message purement informatif de ta part exige-t-il une réponse ? quel délai de grâce ?
   *Ce qui les rend urgents :* ton message du 27/08 18:31:19 portait une échéance de **60 secondes**
   et est passé `timed_out` à 18:32:20. **Tu as attendu huit heures.** Une propriété naïve aurait été
   *satisfaite* pendant toute ton attente.
2. **Le déploiement.** Trois correctifs livrés, aucun en service : `16be24f` (ta boîte),
   `session-060` tête `9c6f47d` (identité d'émetteur, ta priorité du 08h30, livrée par rc7-flux avec
   deux témoins), et `session-058-persistance-annuaire` têtes `8ebe89a`/`ab8960f` (livrée par jc1-flux,
   cinq témoins, cinq mutants).
3. **Les dix agents tmux.** **Sept vivacités attestées** — leur compteur de silence est retombé de
   ~6000 s à moins de 25 s après sollicitation. **Trois inconnues** : `essai-claude-distant`,
   `essai-distant`, `rc1`. Ce ne sont pas dix contextes vivants : c'est sept processus qui répondent
   et trois dont je ne sais rien. Je n'arrête rien.

## Ce qui a avancé sans toi

Les dix cartes de reprise sont écrites et intégrées dans `main`. Le registre est réparé et rend
453 constats. Un témoin de persistance est posé **et éprouvé** — il répond. Le lot orphelin, signalé
sans réclamant depuis 36 h, a enfin un mandat de **jugement** — juger n'est pas intégrer, et
personne ne peut trancher l'intégration d'un lot que personne n'a lu.

Une soixantaine de constats ont été inscrits aujourd'hui, dont une vingtaine me réfutent.
