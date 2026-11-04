# Session 232 — Capacité des copies et diagnostic de reconnexion

Autorisation : François, 4 novembre 2026. Correction et augmentation des
plafonds demandées explicitement, sans nouvelle validation intermédiaire.

## Besoins

Le dépôt opus2D au commit dcd7356f contient 32276 fichiers et 612886253 octets
sélectionnés pour une vue. Le plafond de512Mio bloque donc une revue légitime.
Passer à32Gio, inventorier64Mio, manifeste64Mio et quota des métadonnées worker
2Gio afin de préserver la capacité simultanée. Le stockage disponible et les
preuves du commit exact restent vérifiés. Les opérations de copie et de contrôle
disposent de30minutes chacune ; les autres délais Git et les échéances des
missions admises ne changent pas. Aucune tâche existante n'est rejouée.

Chaque nouveau dépassement mesuré nomme son objet, une borne basse du volume
observé et le plafond exact. Les anciens reçus génériques restent lisibles et
explicitement non mesurés. Pas de chemin ou donnée source dans le diagnostic.

Le second incident concerne les refus de preuve et la remise orpheline après
une coupure du pont. Les contrôles natifs Bridget appellent déjà l'attestation.
L'appel manuel est une lecture et ne change pas les droits. Conserver les
motifs connus de transport au lieu d'annoncer un changement de droits.

## Conservation et activation

Préserver la mission702d272e admise, les tâches7024 et77ba483e, leurs clés et
les travaux métierT3. Aucune annulation, remplacement ou migration de moteur.
Une image de retour doit comprendre les nouvelles bornes et les nouveaux
reçus. L'ancien retour0.8/r4 ne suffit pas pour une vue de plus de512Mio.
Livrer et charger les composants concernés, faire une revue indépendante,
pousser les sources et nettoyer seulement les copies qualifiées.
