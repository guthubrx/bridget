#210 — Refus de préparation exploitables
Correction autorisée. Les refus R53 et R54 perdent la cause Git dans WorkspacePrepareFailed.
Conserver un diagnostic fermé, corrélé à la même commande et persistant dans l'erreur de tâche.
Aucun stderr, chemin privé ou texte du dépôt dans le diagnostic. Aucun nouveau droit ou rejeu.
Les anciens pairs gardent leurs trames. Les anciens refus restent explicitement inconnus.
Le chargement du correctif ne relance aucune tâche annulée.
