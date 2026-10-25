# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/quickstart.md](../../../../../../specs/181-questions-mobile/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Démarrer l’émulateur Bridget_API35 (ou API36 pour compatibilité) et installer le paquet de recette signé. Associer un appareil de recette au backend isolé.
2. par le parcours natif Bridget, publier une question native depuis un producteur terminal ; vérifier les textes et choix sur Android.
3. Choisir, compléter, agrandir, réduire, fermer/réouvrir ; vérifier le brouillon.
4. Dicter ou injecter un résultat vocal contrôlé, relire et envoyer ; le producteur doit recevoir une seule réponse structurée.
5. Question suivante : application en arrière-plan, réponse notification par choix puis texte libre ; vérifier la confirmation native.
6. Répondre depuis le terminal et vérifier la clôture Android ; rejouer une ancienne action, changer de périmètre et vérifier les refus.
7. Couper le réseau au moment de l’envoi puis reprendre la même opération ; vérifier l’absence de doublon.
8. Vérifier l’adaptateur T3 sur une demande contrôlée. Ne pas simuler de touches dans un fournisseur ni interrompre les vrais fils.
9. Captures originales et manifestes hors Git ; HTML de preuve autonome dans preuves-corrections. Tracer les limites du moteur vocal, des notifications ou d’un mode terminal indisponible.

<a id="extrait-2"></a>

## Extrait 2

Les sessions gérées Codex app-server et Claude stream-json exposent leurs callbacks structurés. Dans un terminal TUI opaque, Bridget ne simule pas de touches et ne capture pas une boîte native qui ne lui est pas exposée : utiliser l’outil MCP Bridget. T3 est un adaptateur facultatif, branché sur runtime-request.respond. Aucun partage de brouillons, protocole ou serveur T3 n’est requis par le noyau ou Android.

<a id="extrait-3"></a>

## Extrait 3

T3 n’a pas de commande terminal équivalente à ce commandant humain ; les garde-fous réutilisent le client local Bridget et sa garde de terminal. La forme et la corrélation des questions reprennent les contrats observés dans l’implémentation T3, avec les différences imposées par le socket local et les permissions d’appareil.
