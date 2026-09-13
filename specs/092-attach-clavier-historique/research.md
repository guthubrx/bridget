# Recherche ciblée — 2026-09-06

## Clavier : décision

L'ancien code ne reconnaît qu'ESC suivi d'Entrée ; les flèches laissent actuellement
leur suffixe dans le texte. Réutiliser l'entrée existante avec décodage incrémental.
Le protocole Kitty permet une désambiguïsation temporaire et empilée. iTerm2
recommande ce protocole aux applications plutôt que sa préférence CSI u globale.
Un CR identique pour deux touches reste indiscernable : pas de promesse sur ces terminaux.

Sources primaires lues :
- https://sw.kovidgoyal.net/kitty/keyboard-protocol/
- https://iterm2.com/documentation-csiu.html

Choix : push/pop temporaire, maintien Alt+Entrée, prise en charge des bytes CSI u
et modifyOtherKeys reçus. Rejet : changer les préférences de l'utilisateur, introduire
un framework TUI ou une dépendance clavier pour ce seul fichier.

## Tests : décision

Les contrôles se testent par comportement visible : contenu envoyé, absence d'envoi,
restauration de terminal, retour au brouillon. Une majorité de tests rapides, complétée
par une couture PTY, évite de remplacer la logique par des mocks de la même logique.
La frappe physique dans iTerm2 reste un niveau distinct d'une injection PTY.

Sources primaires lues selon baseline 08-testing-quality :
- https://testing.googleblog.com/2014/04/testing-on-toilet-test-behaviors-not.html
- https://martinfowler.com/articles/practical-test-pyramid.html

Annuaire de préparation : seul l'équipier de développement Agent (23), Sol/high,
dispose du couloir existant autorisé. Les autres agents sont Codex également ; pas
de revue cross-provider prétendue. Le modèle courant n'est pas modifié.
