# Session 092 — Clavier et historique attach

Date : 2026-09-06. Statut : US4 et US5 installées ; validations dans verification-us4.md et verification-us5.md.
Demande : Shift+Entrée pour une nouvelle ligne, Haut/Bas pour rappeler les saisies,
et mode d'emploi du changement de modèle existant. Base : ea52cd04 (091 clôturée).
Complément US4 autorisé le 06/09 par « go » : édition au curseur et Shift+Entrée seul.
Ce complément rouvre seulement le clavier ; le redimensionnement 093 reste en pause.
Le pilote rédige les artefacts et contrôle ; l'équipier implémente, sans commit automatique.

## Parcours utilisateur

### US1 — Composer plusieurs lignes (P1)

Dans attach TTY, saisir `bonjour`, Shift+Entrée, puis `suite` laisse deux lignes
dans le brouillon, sans message transmis. Entrée transmet exactement un message
`bonjour\nsuite`. Depuis l'amendement US4, Alt+Entrée est ignoré. Un terminal incapable de
distinguer Shift+Entrée d'Entrée n'est jamais présenté comme compatible.

### US2 — Rappeler une saisie (P1)

Après l'envoi de A puis B, saisir un brouillon C : Haut affiche B, puis A ; Bas
revient à B, puis à C inchangé. Rappeler ne transmet rien. Modifier un rappel ne
réécrit pas A/B. L'historique appartient seulement à cette ouverture d'attach.

### US3 — Garder les contrôles et l'affichage (P1)

Le fond gris suit la hauteur de saisie, le statut coloré reste dessous et le
journal continue pendant la navigation. `/model <modèle> <effort>` conserve le
contrôle natif de 091, sans prompt, relance ni extension des droits. Ctrl-C détache.

### US4 — Corriger le brouillon au curseur (P1)

L'humain veut revenir dans le texte avec Gauche/Droite et Option+Gauche/Droite,
puis insérer ou effacer à cet endroit, sans envoyer le message ni perdre la suite.
Priorité : l'absence de ces mouvements empêche la correction quotidienne des saisies.
Test indépendant : saisir `bonjor monde`, reculer avant `r`, insérer `u`,
envoyer ; le corps doit être exactement `bonjour monde`.

Scénarios :
1. Étant donné un brouillon UTF-8, Gauche/Droite déplacent un curseur borné sans
   modifier les octets ni émettre de Send ; insertion et retour arrière agissent là.
2. Étant donné `un  deux trois`, Option+Gauche saute les blancs puis le mot
   précédent ; Option+Droite saute les blancs puis le mot suivant. Un mot est ici
   une suite de caractères non blancs, ponctuation comprise ; LF sépare les mots.
3. Shift+Entrée insère LF au curseur ; Entrée envoie le tampon entier. Option+Entrée
   et Échap puis Entrée sont consommés sans LF ni envoi accidentel.
4. Haut/Bas restent l'historique : un rappel place le curseur à la fin ; le
   retour au brouillon restaure le texte ET sa position. Une édition ne modifie
   jamais une entrée d'historique. Une reconnexion conserve cet état d'invocation.
5. Avec texte multiligne/CJK/emoji, le curseur visuel suit les cellules et les
   lignes du brouillon, pendant les mises à jour du journal, sans écraser le statut.

### US5 — Utiliser les raccourcis classiques et le LF d'iTerm (P1)

Mandat humain : « shift entrée ça envoie le message ; mets-moi tous les mêmes
raccourcis que ceux ci-dessus dans le tableau ». La règle iTerm GlobalKeyMap
Shift+Return envoie `\\n` ; l'humain confirme un envoi indésirable. Le défaut
provient du traitement CR/LF identique et de la conversion ICRNL laissée active.
Ne pas changer les préférences iTerm pour corriger attach.

Scénarios d'acceptation indépendants :
1. Dans un vrai double-TTY, CR (Entrée) envoie ; LF (raccourci Shift+Entrée iTerm
   existant) insère un LF au curseur sans Send. Les formes CSI-u/modifyOtherKeys
   Shift restent admises, Option+Entrée reste inerte. Hors double-TTY, CR/LF
   conservent l'envoi historique. La garde restaure exactement termios à la sortie.
2. Ctrl+A/E placent au début/à la fin de la ligne logique délimitée par LF,
   indépendamment des replis visuels. Ctrl+U/K suppriment respectivement jusqu'au
   début/jusqu'à la fin de cette ligne, sans avaler son séparateur LF ; sur une
   portion vide, aucun effet. L'invite `> ` n'appartient jamais au tampon.
3. Option+Backspace et Ctrl+W suppriment la même plage que le saut par mot gauche
   US4 ; Option+D supprime la plage du saut droit. Ponctuation non blanche incluse,
   sémantique US4 conservée, Unicode jamais coupé au milieu d'un caractère.
4. Ctrl+Y réinsère au curseur le dernier texte supprimé par ces commandes de mot
   ou de ligne. Une suppression effective remplace le registre précédent ; une
   suppression vide ne le change pas. Retour arrière ordinaire ne remplace pas ce
   registre. Pas de concaténation implicite ni accès au presse-papiers système.
5. Tous ces contrôles n'envoient rien ; ils passent par le tampon/snapshot/renderer
   existants. Historique, statut, reconnexion et mode redirigé sont préservés.

## Exigences fonctionnelles

- FR-001 : Shift+Entrée insère LF sans envoi lorsqu'une séquence distincte est
  attestée ; Entrée simple envoie. Amendement US4 : Alt+Entrée/Échap puis Entrée
  sont ignorés, sans envoi. La saisie de texte multiligne collé reste distincte.
- FR-002 : activer le mode clavier enrichi uniquement pendant attach TTY, avec
  restauration à toute sortie contrôlée (Ctrl-C, EOF, erreur). Aucun réglage global,
  aucune négociation bloquante, aucun changement non-TTY ; TERM=dumb reste dégradé.
- FR-003 : décoder les séquences clavier par flux, même réparties sur plusieurs
  lectures ; les contrôles reconnus ne deviennent jamais du texte. Séquence
  inconnue ou trop longue : ignorée, état borné, contrôles d'arrêt toujours possibles.
  UTF-8, effacement, Ctrl-C et Ctrl-D fonctionnent aussi sous encodage enrichi.
- FR-004 : Haut/Bas rappellent les saisies chronologiquement ; bornes saturantes,
  historique vide sans effet, brouillon original restauré après le dernier rappel.
  Une édition quitte la navigation et devient le nouveau brouillon.
- FR-005 : mémoriser les messages et commandes `/model` valides après écriture
  réussie sur la connexion seulement. Vide, commande invalide ou écriture échouée
  non mémorisés. L'historique atteste une saisie émise, pas une livraison acquittée.
  Pas de déduplication implicite : deux envois identiques restent deux entrées.
- FR-006 : historique volatile, 100 entrées et 1 Mio de corps UTF-8 au maximum,
  éviction des plus anciennes. Une saisie dépassant seule 1 Mio peut être envoyée
  mais n'est pas conservée. Ni disque, ledger, fichier fournisseur, ni synchronisation.
  Il survit à une reconnexion socket dans la même invocation, pas à sa fermeture.
  La navigation est réservée au vrai TUI (stdin ET stdout TTY) : aucune séquence
  de remplacement n'est émise dans une sortie redirigée ; flèches ignorées sans
  polluer le texte dans ce mode dégradé.
- FR-007 : réutiliser le renderer 091 et les points d'envoi/contrôle existants ;
  préserver repli multiligne, Unicode, redimensionnement, NO_COLOR et statut attesté.
- FR-008 : documenter FR/EN et skill active : touches, portée volatile, limites
  terminal et commande `/model` existante. Aucun nouveau protocole Bridget/MCP.
- FR-009 : curseur dans InputBuffer, borné entre zéro et longueur UTF-8 ; déplacements
  et retour arrière sur des caractères Unicode entiers (socle scalaire existant,
  aucune promesse supplémentaire de segmentation en graphèmes). Ne jamais découper
  les octets d'un caractère ; insertion fragmentée UTF-8 au milieu vérifiée.
- FR-010 : décoder Gauche/Droite CSI/SS3 ; Option+flèche CSI 1;3D/C et formes
  ESC b/f usuelles. Les équivalents CSI-u attestés au contrat sont admis. Ne pas
  absorber les flèches normales dans les variantes par mot ; contrôles inconnus ignorés.
- FR-011 : renderer reçoit un instantané cohérent texte/curseur ; la fenêtre de
  saisie multiligne garde le curseur visible et laisse le statut sous la zone.
  Hors double TTY, ne pas produire de mouvements ANSI ni prétendre permettre
  l'édition au milieu : conserver l'entrée simple et consommer les flèches.
- FR-012 : amendement US5 à la règle CR/LF : distinguer CR (Send) et LF (InsertNewline)
  seulement avec stdin ET stdout TTY ; désactiver les conversions CR/LF dans la
  garde correspondante et restaurer les attributs d'origine sans modifier le reste.
  Pipes et stdin TTY/stdout redirigé conservent CR/LF Send. Ctrl-J, même octet LF
  que le raccourci iTerm, insère aussi une ligne en TUI : aucun nom de touche inféré.
- FR-013 : implémenter Ctrl+A/E/U/K/W/Y et Option+Backspace/D selon US5, formes
  brutes et CSI-u explicitement listées au contrat. Réutiliser les bornes de mot
  US4, aucune copie parallèle des règles de découpage. Hors double-TTY, consommer
  les nouveaux contrôles d'édition sans altérer le tampon.
- FR-014 : conserver un seul dernier fragment supprimé par les commandes US5,
  volatile à l'invocation attach (reconnexion conservée, nouvelle instance vide).
  Stockage proportionnel à ce seul fragment, pas de pile ni duplication à chaque
  frappe ; aucun fichier/presse-papiers système/protocole/agent concerné.

## Critères de succès

- SC-001 : fixtures CSI u et modifyOtherKeys pour Shift+Entrée, injectées octet
  par octet dans l'entrée réelle : aucun Send avant Entrée, puis un corps LF exact.
- SC-002 : A/B/brouillon puis Haut/Haut/Haut/Bas/Bas/Bas, édition, message multiligne
  et UTF-8 : résultat exact et zéro envoi lors de la navigation.
- SC-003 : franchir 100 entrées et 1 Mio, tester entrée géante, échec d'écriture,
  commande invalide et nouvelle instance : bornes et absence de persistance prouvées.
  Réutiliser l'état d'invocation après reconnexion et retrouver l'historique ;
  instance d'invocation différente vide. Sortie redirigée sans rappel/ANSI.
- SC-004 : pseudo-TTY existant : activation/restauration symétrique sur succès,
  Ctrl-C/EOF et erreur ; non-TTY/dumb sans séquence d'activation. Tester entrée
  fragmentée, séquence inconnue/surdimensionnée et récupération des contrôles.
- SC-005 : tests attach existants verts ; rappel et Shift+Entrée préservent rendu
  multiligne, largeur étroite, couleurs/reset et `/model` contrôle sans Send.
- SC-006 : recette clavier reproductible et synthèse des validations indiquant
  séparément bytes injectés sur PTY et éventuelle frappe physique observée.
- SC-007 : fixtures au vrai handle_input_byte, fragmentées : gauche/droite, sauts
  par mot, bornes, édition au milieu et LF au curseur ; zéro Send avant Entrée,
  corps final exact. Option+Entrée sous trois encodages ne transmet ni n'insère LF.
- SC-008 : historique avec brouillon à curseur non final, édition d'un rappel,
  reconnexion, UTF-8 fragmenté, lignes multiples et sortie redirigée.
- SC-009 : PTY et grille d'un émulateur réel, coordonnées du curseur vérifiées
  avant/après insertion, retour arrière et flux journal ; lignes étroites, CJK et
  bord droit exact. Ne pas confondre vérification d'octets ANSI et grille effective.
- SC-010 : oracle rouge puis vert sur PTY avec ICRNL initialement actif : bytes CR
  et LF restent distincts après garde, LF n'écrit aucune trame, CR émet exactement
  le tampon multiligne ; restauration après Ctrl-C/EOF/erreur. Test inverse redirigé.
- SC-011 : matrice de toutes les commandes US5, ligne médiane/LF vide/texte vide,
  mots Unicode/ponctuation, Ctrl+Y après suppression puis navigation/historique ;
  zéro Send avant CR, édition d'un rappel sans mutation de l'historique.
- SC-012 : grille attach réelle avec les séquences correspondant aux réglages
  iTerm relevés ; texte et curseur exacts, saisie en cours sans effet sur le journal.
  La frappe physique humaine finale reste explicitement distinguée de l'injection.

## Hors périmètre

Pas d'éditeur complet (sélection, recherche, completion, raccourcis non demandés), pas
d'historique intersessions, pas de nouveau modèle sélectionné pour l'équipier,
pas de modification fournisseur/sandbox/daemon, pas de déploiement ni commit automatique.
Pas de nouvelle dépendance. Si Entrée et Shift+Entrée sont les mêmes octets,
aucun délai ou inférence ne permet de les distinguer : limite documentée, sans
raccourci alternatif imposé. Le correctif resize 093 n'est pas déclaré résolu.
