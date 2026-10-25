# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/042-appariement-par-origine-prouvee-et-entretien.md](../../../../../decisions/042-appariement-par-origine-prouvee-et-entretien.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le pont T3 relaie à l'expéditeur la réponse du tour qu'a déclenché sa demande. T3 ne lie pas un
message utilisateur au tour qu'il lance : l'appariement se faisait par rang, le k-ième message
utilisateur répondant au k-ième tour assistant de la fenêtre lue, et seulement si les deux décomptes
étaient égaux. Sur les données réelles, cette égalité est l'exception :

<a id="extrait-2"></a>

## Extrait 2

Seize demandes suivies ont ainsi été abandonnées depuis le 14/09, destinataire ayant pourtant répondu.
Les points de contrôle de T3, envisagés comme décompte fiable, ont été écartés sur mesure : absents
hors dépôt Git, et sans correspondance un pour un avec les messages sur la plupart des fils.

<a id="extrait-3"></a>

## Extrait 3

1. **Preuve directe d'abord.** Le pont relevait déjà, pour le journal d'observation, l'origine du
   dernier tour : le message utilisateur unique dont l'horodatage égale strictement l'horodatage de
   demande du tour, T3 projetant les deux de la même commande. Cette table est désormais alimentée à
   chaque lecture et consultée en premier : la réponse est le texte de CE tour, et seulement de lui.
   Le calcul par rang reste en repli, inchangé, quand l'origine n'a pas été relevée.
2. **Un tour prouvé muet est annoncé**, après cinq lectures concordantes pour absorber le décalage
   de projection de T3, par un texte explicitement attribué à Bridget.
3. **Entretien horaire du daemon**, premier passage deux minutes après le démarrage : remises
   expirées encore « en vol » passées en sort inconnu ; envois expirés depuis plus de 30 jours purgés
   (les lancements d'équipiers, liés en cascade au même enregistrement, sont épargnés) ; état
   d'identité des processus disparus retiré ; temporaires et configurations MCP orphelins ramassés ;
   journaux de service au-delà de 20 Mio tournés, une génération conservée.
4. **Worktrees fusionnés** retirés par le script de construction, sous les conditions de la
   constitution (XVI.3).
