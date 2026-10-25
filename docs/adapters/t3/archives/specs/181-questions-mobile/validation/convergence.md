# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/validation/convergence.md](../../../../../../../specs/181-questions-mobile/validation/convergence.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Fallback manuel, primitive converge absente du catalogue. Lecture spec/plan/tasks/checklist et réalisations, puis confrontation aux tests et recettes. Résultat CONVERGED ; aucune écriture pendant la comparaison. SHA256 tasks.md avant/après :44229c6aead226def7edb4d3274f99fe32bfc84829cf3ad78032d577b369e7f6. Les limites de voix réelle et d’application T3 ne sont pas assimilées à une preuve de leur runtime.

<a id="extrait-2"></a>

## Extrait 2

|Exigence|Réalisation et preuve observée|
|---|---|
|FR001/002|daemon/user_input.rs:13,32,91,137 : source attestée/instance/projet, refus humain/agent ; test authority_separates_producer_human_and_foreign. mobile_questions.rs:268 contrôle appareil ; mobile_scope_revision_replay_and_receipt.|
|FR003|user_input.rs:34,102 valide et conserve ; UserInputForm.kt:25,117 options/descriptions et champ libre ; tests preserves_options_and_free_complement/multiple_and_text_capability, JVM et recette9b5a8813.|
|FR004|UserInputForm.kt:55 repli/agrandissement/navigation ; MainActivity.kt:406 volet au-dessus saisie ; captures181-18/03/06/07.|
|FR005|user_input.rs:225,325 état consigné puis ACK ; UserInputForm.kt:83/145, MobileViewModel.kt:687/731 état local/inconnu/reçu ; confirmation capture181-08.|
|FR006|UserInputForm.kt:99 résultat vocal lié au qid, champ éditable/choix conservés ; UserInputVoiceTest contrôlé PASS et capture181-11 ; reconnaissance audio réelle non qualifiée.|
|FR007|UserInputNotifications.kt:31/62/85 relecture auth, options/RemoteInput, commande avant réseau ; MainActivity.kt:127 ouvre conversation source ; notification réelle et coupure réseau PASS.|
|FR008|user_input.rs:225 transactionImmediate/enveloppe immuable ; response_replay_revision_and_competing_answer et recette375371d5 même commande après coupure.|
|FR009|daemon/user_input.rs:13/432/610 capacité/permissions/corrélation ; test mobile change projet/révocation, tests callbacks/transport ; aucune réinjection prompt.|
|FR010|MobileViewModel.kt:195 et704 rafraîchissement5s/show anciennes ; LocalStore.kt:149 brouillon lié appareil/révision ; réponseTTY et fermeture réelles.|
|FR011|MCP bridget_ask_user mcp.rs:2531/9057 ; cli.rs:12358 ; domaine central user_input sans importT3 ; namespaceQA natifBridget et réponseTTY confirmée.|
|FR012|Régression155 :62PASS puis cas isolé corrigé1PASS ; anciens handlersQuestions/File maintenus ; garde session fournisseur ne confond pas permissions et questions.|
|FR013|AVD API36/API35, reçus/captures privés, diaporama vérifié ; livraison reste une phase suivante, pas présumée.|

<a id="extrait-3"></a>

## Extrait 3

La lecture de l’auteur a relevé une éviction à capacité32 de l’adaptateur T3. Ce défaut n’est pas issu de Converge ni d’un verdict indépendant. T023 ajoutée et exécutée en phase Audit fix. Test avant :31au lieu de32, assertion utile atteinte. Test après et lot181 :18PASS. Relire le delta et vérifier tasks inchangé avant/après au passage2. Aucune extension de périmètre.

<a id="extrait-4"></a>

## Extrait 4

Passage2 : CONVERGED après relecture du delta T3/capacité et des exigences/contrats conservés. Aucune tâche ajoutée ; SHA256 tasks avant/après f8ed38939a6814ac92a8ef7b4385793e3d939400420ad7b6be9746e36e03868e. Cette consignation suit la comparaison, sans écriture pendant celle-ci.
