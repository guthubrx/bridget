# Données locales 092

État privé par attach, jamais sérialisé sur disque ou au daemon :

- InputBuffer.bytes : brouillon UTF-8 courant (existant).
- Décodeur : texte/ESC/CSI/SS3/ignorance d'une séquence trop longue ; accumulation
  bornée à 32 octets, arrêt toujours possible. Remplace le booléen alt_prefix.
- Historique : VecDeque de corps émis, 100 entrées et 1 048 576 octets maximum ;
  compteur d'octets si nécessaire à la borne, invariant testé contre la somme réelle.
- Navigation : indice optionnel et brouillon de retour optionnel seulement pendant
  la navigation. Haut capture le brouillon une seule fois ; Bas au-delà du plus récent
  le restitue. Éditer quitte la navigation sans modifier les entrées.
- Garde clavier : indique si attach a effectivement ouvert sa pile de mode, afin
  de ne fermer que celle-ci une seule fois. Termios garde son autorité existante.

Pas de changement du modèle RuntimeSelection, AttachClientState de transport,
ledger, protocole ou schéma. Aucune migration.
