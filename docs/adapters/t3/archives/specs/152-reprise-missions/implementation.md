# Archives techniques de l’adaptateur T3

Document d’origine : [specs/152-reprise-missions/implementation.md](../../../../../../specs/152-reprise-missions/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

H6 distingue l’installation sans relance des sessions 147 à 151, référence
historique, de l’activation de production coordonnée par le principal sans
interrompre les autres fils T3. Le quickstart affirme encore qu’aucune relance
n’est faite par cette session et réserve l’activation à l’utilisateur.
Ces formulations ne sont pas équivalentes au dernier contrat de livraison en
production. Le principal vérifiera les processus et travaux actifs lors de
la livraison. Ce reçu n’ajoute aucune interdiction ni nouvelle approbation,
et ne tranche pas l’activation à sa place.

<a id="extrait-2"></a>

## Extrait 2

Le dernier contrat utilisateur demande une livraison de production fonctionnelle.
H6 confie l’activation coordonnée au principal sans interrompre les autres fils
T3. L’installation sans relance des sessions précédentes reste historique.
Sur décision explicite du principal, l’ancienne phrase du quickstart réservant
l’activation à l’utilisateur est remplacée par le contrôle réel après activation
coordonnée. Le principal observe processus, namespace et quiescence en phase 8 ;
aucun redémarrage T3 ni nouvelle approbation n’est ajouté.

<a id="extrait-3"></a>

## Extrait 3

| Question XIX/XX | Réponse du reviewer, résumée |
|---|---|
| Nécessité | RED compilés des défauts prévus ; pause consommée après request_unavailable ; fixtures Working avec admission durable. |
| Simplicité/réemploi | ADR027, garde, magasins, continuations, flotte/droits, arrêts et entretien existants ; deux usages réels par extraction ; table de refus requise par FR-021/023. Aucun module/dépendance/outil public nouveau. |
| Hypothèses | Données lisibles, identité et droits attestés, journal de pause autoritaire ; borne2, bail soumis300s non gelé ; FR-029 limitée au natif lisible, refus sûr cible/CAS/stockage. |
| Vérifié | Diff entier, ADR/implementation, gardes, compteur sauvegardé avant effet, trois correctifs finaux et cinq F2 ; SHA sources conformes ; R8 personnellement lu295/0/1, sept RC0. |
| Non vérifié | Modèle réel, T3, activation, benchmark, couverture/mutation ; aucun Cargo/Git/service personnel ; R1 antérieur au patch, solo098 vérifié par principal/docowner. Aucun GREEN global ni Converge/audit. |
| Conservé/supprimé | Dates/champs durables, interruptions, droits/identité/CAS/ACK151 et oracles stricts conservés ; helper150/trigger mort déjà retirés, vrai UPDATE refusé gardé ; cinq adaptations149 et témoin inchangé ; zéro suppression additionnelle prouvée. |
