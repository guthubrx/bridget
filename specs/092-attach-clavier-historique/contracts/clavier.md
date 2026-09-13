# Contrat local du clavier attach

Ce contrat ne modifie aucune trame Bridget. Corps conservés en UTF-8 exact.

| Entrée filaire | Action |
|---|---|
| CR simple ; CSI 13 u ; CSI 13;1u | envoyer |
| LF simple / Ctrl-J | double-TTY : insérer LF ; sinon envoyer (US5) |
| ESC CR/LF ; CSI 13;3u ; CSI 27;3;13~ | ignorés (Alt+Entrée retiré par US4) |
| CSI 13;2u ; CSI 27;2;13~ | insérer LF (Shift+Entrée) |
| CSI A / CSI B ; SS3 A / SS3 B | historique précédent/suivant |
| CSI 1;1A / CSI 1;1B | même action sans modificateur |
| Ctrl-C brut ou CSI 99;5u | détacher |
| Ctrl-D brut ou CSI 100;5u | détacher si saisie vide |
| CSI 27u / CSI 27;1u (Échap enrichi) | préfixe ; suivi d'Entrée ignoré |
| BS/DEL brut ou CSI 127u / CSI 127;1u | effacer le caractère avant le curseur |
| CSI D/C ; SS3 D/C ; CSI 1;1D/C | gauche/droite d'un caractère |
| CSI 1;3D/C ; ESC b/f | gauche/droite d'un mot |
| Ctrl-A/E bruts ; CSI 97;5u / CSI 101;5u | début/fin de ligne logique |
| Ctrl-U/K bruts ; CSI 117;5u / CSI 107;5u | effacer avant/après jusqu'à la limite LF de ligne |
| Ctrl-W brut ; CSI 119;5u ; ESC BS/DEL ; CSI 127;3u | effacer mot précédent |
| ESC d ; CSI 100;3u | effacer mot suivant |
| Ctrl-Y brut ; CSI 121;5u | réinsérer le dernier fragment supprimé US5 |

CSI = ESC `[`, SS3 = ESC `O`. Les formes non reconnues n'ajoutent pas leur
suffixe `[...` au message. Un Ctrl-C/EOF ne peut rester emprisonné dans une
séquence incomplète. Une pression enrichie non listée ne devient pas une commande
devinée ; préserver le texte UTF-8 ordinaire et normaliser les contrôles réellement
activés par le seul mode de désambiguïsation. Ne pas activer le mode « toutes touches ».
Les notifications de relâchement de touche ne déclenchent jamais un deuxième envoi.

Vérification amont du 06/09 : la table officielle Kitty encode LEFT/RIGHT par
CSI 1 D/C, avec modificateur ajouté, pas par des numéros CSI-u privés déduits.
Source : https://sw.kovidgoyal.net/kitty/keyboard-protocol/#functional-key-definitions
Ne pas ajouter de variantes numériques sans producteur attesté.

Activation TTY : push `ESC[>1u`, restauration pop `ESC[<u`, symétriques et
idempotentes avec la garde, y compris erreur après activation. Un terminal ancien
ignore ce protocole ; Shift+Entrée n'y est pas garanti. TERM=dumb et non-TTY
ne reçoivent aucun push/pop. NO_COLOR coupe les styles, pas les fonctions clavier.
Ne pas activer modifyOtherKeys globalement : on accepte sa séquence Shift+Entrée
si le terminal la fournit déjà, sans changer son état non empilé.

US5 : en double-TTY, la garde désactive ICRNL/INLCR/IGNCR afin de conserver les
octets CR/LF réellement fournis. Ce choix ne dépend pas du succès du mode Kitty :
il sert aussi au raccourci iTerm SendText `\\n`. TERM=dumb ne reçoit toujours aucun
push/pop ni couleur. Tout termios est restauré exactement ; hors double-TTY les
conversions existantes sont conservées. L'amendement n'identifie pas une touche
physique d'après LF : Ctrl-J a nécessairement la même action en TUI.

Les nouvelles commandes d'édition US5 sont inertes hors double-TTY. Pour les
variantes CSI-u, événements press/repeat seuls ; release/inconnu ignorés. En
séquence ESC CR/LF, l'exception Option+Entrée inerte précède la règle LF simple.
Ctrl-U/K excluent toujours le séparateur LF ; au début/à la fin de ligne : no-op.
Les suppressions mot réutilisent exactement la table US4 ci-dessous. Le registre
Ctrl-Y ne change que sur suppression US5 non vide, survit à navigation/envoi et
reconnexion, n'est pas modifié par backspace ordinaire, n'est jamais publié hors
du message que l'humain enverra volontairement. Un Ctrl-Y répété insère à nouveau
le même fragment ; aucune consommation ni concaténation de fragments supprimés.

Historique : messages et contrôles `/model` valides écrits avec succès, ordre
chronologique ; état de réception distinct. Un rappel n'est ni un envoi ni une
nouvelle entrée. Un message géant exclu de l'historique n'évince pas les précédents.
Le brouillon édité remplace le brouillon de retour ; les entrées restent immuables.

## Positions US4

Les positions sont des offsets UTF-8. Gauche par mot saute les blancs à gauche,
puis les non-blancs à gauche ; Droite saute les blancs à droite, puis les
non-blancs à droite. Table exacte sur `un  deux trois` (longueur14) :

| Départ | Option+Gauche | Option+Droite |
|---|---|---|
| 0 | 0 | 2 |
| 2 | 0 | 8 |
| 3 | 0 | 8 |
| 4 | 0 | 8 |
| 6 | 4 | 8 |
| 8 | 4 | 14 |
| 9 | 4 | 14 |
| 14 | 9 | 14 |

Snapshot sous verrou unique : texte exact + cursor_byte. Un caractère UTF-8
fragmenté en entrée est assemblé dans au plus4 octets avant insertion atomique ;
avant sa complétion, le brouillon visible et son curseur ne changent pas.
Ctrl-C reste opérant pendant une séquence incomplète. Au bord droit exact, ne pas
inventer une ligne de saisie supplémentaire : représenter la marge terminal puis
insérer correctement au prochain caractère. Tester aussi avant/après LF, LF final
avec ligne vide, CJK/emoji, et conservation du statut après redessin.
