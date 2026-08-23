# Implémentation 013 — Vue attach lisible

## SC-001 — Densité en 80×24

Le test `sc001_trois_tours_deterministes_tiennent_dans_un_ecran_80_par_24`
injecte dans le renderer de production un corpus journal v1 déterministe de
trois tours complets, dont un appel d'outil nommé `Read src/main.rs`. Il compte
les lignes de l'ancien rendu événement-par-événement puis les lignes visuelles
des blocs compacts à 80 colonnes. Un budget global de 2 secondes rend tout
blocage falsifiable.

- ancien rendu : **67 lignes** ;
- rendu compact : **13 lignes visuelles** ;
- contrainte : **13 ≤ 24**, contre une baseline **67 > 60**.

Le chemin réel journal→abonnement→deux vues reste couvert par les bancs 008,
rejoués sans modification. La suite correspondante est verte :
`sc001_append_vers_rendu_attach_reel_reste_sous_les_seuils_locaux`,
`sc002_rejeu_vers_suivi_traverse_la_rotation_sans_perte_ni_doublon` et
`sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent`
passent ensemble en 90,33 s.

## Finition observable

- un équipier arrêté produit `AgentStopped`, distinct de `AgentUnknown`, et
  seuls les équipiers ACP vivants (`connected`, `busy`, `dnd`) sont listés ;
- les horodatages du journal affichent désormais un suffixe `UTC` explicite ;
- stdout non-TTY conserve le chemin historique, hors delta journal v1 déjà
  versionné pour le vrai titre d'outil ;
- `docs/DEPRECATIONS.md` a été relu : aucun chemin supplémentaire n'est
  déprécié par cette session.

## Fermeture de la revue hostile

- C1 : `RendererSender` transporte explicitement l'état du raw mode ; le test
  pipe/pipe traverse le vrai chemin de saisie et prouve l'absence d'écho ;
- C2/C10 : la géométrie lit `ws_col` et `ws_row`. Seule la queue tenant dans
  `hauteur - 2` reste effaçable ; les rangées sorties sont écrites une fois
  dans le scrollback. Le resize recalcule aussi la hauteur réellement occupée
  avant l'effacement ;
- C3 : les commandes déjà en attente sont drainées en lot et les redraws live
  coalescés ; 31 chunks consécutifs produisent un seul redessin ;
- C5 : l'événement permission complète la dernière ligne d'outil corrélée avec
  `accordé`, `refusé` ou `décidé` ;
- C6/C7 : le bloc consomme sa capacité utile de 64 Kio par préfixe accepté et
  n'affiche qu'un seul marqueur de troncature ;
- C8 : une reprise sans `turn_start` reprend l'heure réelle du premier
  événement et porte l'étiquette `tour repris en cours` ;
- C9 : la matrice constructeur est complétée par un golden pipe/pipe réel.

Compromis C4 : le canal reste borné et le traitement par lots réduit fortement
la saturation, mais les événements sémantiques ne sont pas abandonnés lorsque
stdout applique une contre-pression noyau (par exemple XOFF). Le lecteur peut
donc attendre le renderer afin de préserver une sortie exacte. Rendre stdout
annulable exigerait un protocole d'écriture non bloquant et une politique de
perte dédiée ; ce changement dépasse le TUI léger et doit être spécifié avant
de modifier la sémantique de livraison.
