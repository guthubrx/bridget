# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-recette-reelle-r2.md](../../../../../../../specs/148-identite-delegation/validation/glm-recette-reelle-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. Opt-in fermé : BRIDGET_148_REAL_GLM doit valoir exactement "1".
   PASS_ZAI_API_KEY n'accepte que absent ou "1". Toute autre valeur échoue.
   Route restreinte à agent_type "glm" + route "existing_gclaude_keychain_command".
   Variable absente de forbidden_env, présente, non vide.
   Sept branches couvertes par le test unitaire sec.
2. Confidentialité : ZAI_API_KEY lue par var_os. Passée uniquement à
   l'environnement du processus daemon (en mémoire). build_environment
   (lifecycle.rs:460) copie par nom via pass_env. Aucune valeur dans le
   registre privé JSON, le reçu, le journal, le prompt ou les arguments.
   Le relais n'archive que des noms de modèles.
3. Contournement du bloqueur R1 : gclaude ligne 8 teste
   [ -z "${ZAI_API_KEY:-}" ] avant d'appeler le trousseau. La variable
   héritée court-circuite security find-generic-password. HOME reste privé.
   Le trousseau n'est plus sollicité.
4. Ownership/nettoyage : OwnedChild::stop vérifie PID, PPID == processus
   courant, exécutable canonique attendu, rejet "firefox". SIGTERM sur PID
   unique. Jamais -9. Jamais de groupe. Zombie attendu puis récolté.
   Drop sans panic ; la recette exige stop() explicite réussi avant le reçu.
   Les tests secs attestent kill(pid,0) == -1 après EOF et après unwind.
5. Recette exacte : un seul appel bridget_delegate. Nonce absent de la
   mission. Une seule réponse corrélée enfant → parent. Une seule ligne
   native_delegations. Modèles annoncés tous égaux au modèle demandé.
   Aucun T3 dans le chemin enfant (t3_used false ; spawn natif via
   lifecycle submit_spawn → superviseur géré).
6. Préconditions constatées avant exécution :
   - registre /Users/moi/.cache/bridget-core/agents.json mode 0600 ;
   - forbidden_env glm = [ANTHROPIC_API_KEY] seul (pas ZAI_API_KEY) ;
   - settings /Users/moi/.claude-glm/settings.json sans clé embarquée
     (route gclaude confirmée) ;
   - hash gclaude = dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e ;
   - CLI /Users/moi/.local/bin/claude → versions/2.1.296 exécutable ;
   - ZAI_API_KEY présente dans l'environnement du testeur (présence seule
     vérifiée, valeur jamais affichée ni écrite).

<a id="extrait-2"></a>

## Extrait 2

Identité T3 corrigée : aucun T3 dans le chemin natif enfant. Délégation
native en un appel autonome : un seul bridget_delegate, une tâche, un
enfant, une réponse corrélée. Modèle exact glm-5.3 demandé et annoncé.
Résultat durable : ligne unique native_delegations avec modèle et effort
null (catalogue sans efforts). Sources d'auth/settings/CLI intactes.
Le dépôt n'a pas été modifié par le testeur.
