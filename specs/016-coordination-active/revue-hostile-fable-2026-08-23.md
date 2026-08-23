# Revue hostile de conception — session 016 (2026-08-23 soir)

**Croisement** : relecture par un moteur distinct de celui de l'auteur.
**Objet** : spec.md + plan.md au commit 76be5d1. **Verdict : AMENDER**
(F27 APPROVE sous réserve C8 ; F28 AMENDER — C2 ; F29 AMENDER — C1, C3,
C4, C5a, C6). Architecture réducteur pur + outbox transactionnelle +
lignée par générations jugée saine et fidèle à FR-022/FR-014.

## Socle sain (C11)
Réducteur fermé (5 effets), graphe DÉCLARÉ (refus typés cycle/inter-objectif/
rétroactif), aucune inférence, texte libre inerte, chaîne préautorisée
épinglée avec épuisement → intervention humaine, deux-vérités propres.

## Constats majeurs
- **C1 [F29, bloquant]** Cycle de vie des demandes Bridget inter-générations
  non spécifié : la réassignation doit produire cancel(demande source) +
  nouvelle demande suivie pour le successeur (sa propre échéance — sinon F29
  inapplicable dès la génération 2) + notification au sortant.
- **C2 [F28]** « Terminal qualifiant » ambigu vs règle 6 (vérification avant
  relais) : greffe mécanique du rapport OU clôture évaluée humaine — trancher
  (option : défaut = greffe du hash, drapeau par arête « exiger clôture
  évaluée ») et documenter l'écart assumé.
- **C3 [F29]** Source d'observation des candidats de repli non spécifiée et
  volatile (le guichet n'expose rien sur des tiers) : définir la source
  attestée et l'épingler dans la décision, énoncer le déterminisme
  post-commit — OU v1 pure : admissibilité = faits registre seuls.
- **C4 [F29]** Une réponse n'est pas une livraison : trancher si un answered
  corrélé à une relance suspend/réarme le compteur (fait attesté, pur) ou
  assumer PAR ÉCRIT que répondre ne protège pas.

## Constats moyens
- **C5** Gouvernance : acte automatique DÉFENDABLE (jugement à la déclaration
  de politique) MAIS (a) inhibition humaine requise — l'annulation/clôture
  administrative arrête le compteur (palliatif jusqu'à pause/reprise) ;
  (b) notifier le participant SORTANT de la clôture de sa génération.
- **C6** Arbitrage par lot : au threshold_event, consulter le MÊME lot relevé
  pour un delivery_report corrélé avant de réassigner — ou assumer
  le comportement file-ordre par écrit.
- **C7** Gate d'entrée : contrat 015 mergé/gelé AVANT ouverture du lot A
  (reminder_sent = ajout versionné assumé). Dégradation Gap/Unavailable : RAS.

## Constats mineurs
- **C8 [F27]** Couverture partielle de l'incident fondateur (la clôture
  d'objectif reste un acte manuel) : l'écrire, SC-1607 le rejoue honnêtement.
- **C9** Bornes anti-tempête présentes par construction (RAS, sauf C1).
- **C10** Couloirs : charge asymétrique du lot C ; vérifier que la couture
  FR-1602 (transaction de clôture existante → outboxes F27) est bien dans la
  propriété de B (store.rs).
