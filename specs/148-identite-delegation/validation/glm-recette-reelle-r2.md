# Rapport testeur indépendant GLM — recette réelle R2 — session 148

Date : 2026-10-10 (exécution 07:14, durée recette 55,19 s)
Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/session-148-identite-delegation
Rôle : relecteur et testeur indépendant. Dépôts en lecture seule.

## 1. Verdict global

- Revue de la correction : **APPROVE**, aucun finding bloquant.
- Tests secs : **3 PASS / 0 FAIL / 2 ignorés** (recette réelle + performance_daemon_worker).
- fmt --all --check : **PASS**.
- Recette réelle GLM : **PASS**. Reçu : /private/tmp/ng148-bfcc36f958/receipt.json

## 2. Gel des fichiers — hashes début/fin

| Fichier | Hash (identique avant/après tout) |
| --- | --- |
| crates/bridget-daemon/tests/native_delegation_real_glm.rs | ae3a6aeabffc0a57f51716d0c01158ed6efd6e86036699c3ad7e5ced4246ae55 |
| crates/bridget-daemon/tests/fixtures/native_delegation_real_glm_relay.py | 30c23c649728b72eac5eaab497e3d303d5597df98cb7dcaff175ba484a25787d |

Le hash du relais correspond aussi à l'archive
specs/148-identite-delegation/validation/glm-tests-natifs-r2-complet.md.
Le brief collait les mots ; la transcription initiale avait perdu un caractère.
La comparaison stricte par programme confirme la correspondance exacte.

## 3. Revue — points vérifiés

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

Observation non bloquante : la définition production glm contient
--dangerously-skip-permissions --permission-mode bypassPermissions.
Configuration préexistante de l'utilisateur. La recette ne l'étend pas.
Mission en lecture seule dans un projet isolé sous racine privée 0700.

## 4. Tests secs

Commande (TMPDIR privé /tmp/t148gRDmgHN 0700, umask 077, sans --ignored) :
cargo test -p bridget-daemon --test native_delegation_real_glm

Résultat : 3 passed ; 0 failed ; 2 ignored
(native148_parent_codex_fixture_delegates_once_to_real_glm,
support::performance_daemon_worker). Durée 0,04 s après compilation.
La compilation couvre la recette réelle (latest recipe incluse).
cargo fmt --all --check : exit 0, aucune différence.

## 5. Recette réelle

TMPDIR privé /tmp/t148rXXXXXX 0700, umask 077. Commande exacte :

BRIDGET_148_REAL_GLM=1
BRIDGET_148_REAL_GLM_REGISTRY=/Users/moi/.cache/bridget-core/agents.json
BRIDGET_148_REAL_GLM_AGENT_TYPE=glm
BRIDGET_148_REAL_GLM_MODEL=glm-5.3
BRIDGET_148_REAL_GLM_AUTH_COMMAND_SHA256=dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e
BRIDGET_148_REAL_GLM_PASS_ZAI_API_KEY=1
cargo test -p bridget-daemon --test native_delegation_real_glm -- \
  --ignored --exact native148_parent_codex_fixture_delegates_once_to_real_glm --nocapture

Résultat : 1 passed ; finished in 55.19 s.
ZAI_API_KEY héritée de l'environnement du testeur. Aucune valeur affichée.

### Reçu (chemin absolu)

/private/tmp/ng148-bfcc36f958/receipt.json

- task_id : 505be658-6986-434e-a5db-9142811b275e
- child_agent_id : 9c898d1a-5791-44d1-b7b2-1251ee4d8c9b
- message_id : fe75b47c-ffc1-4757-bbb1-b723a852086c
- glm : réel ; parent : fixture native Codex, aucun modèle Codex exécuté
- requested_model : glm-5.3 ; provider_models : glm-5.3 (6 événements)
- result_correlated : true ; t3_used : false ; cleanup_confirmed : true
- auth_route : existing_gclaude_in_memory_ZAI_API_KEY
- settings_unchanged : true ; command_unchanged : true

Nonce marqueur : GLM_NATIVE_148_665e0fea1270442f95ee65d9890f7a4b
Évidence modèles : 6 lignes provider_model, toutes glm-5.3.

## 6. Sources inchangées après recette

| Source | Hash après (identique avant, attesté par le test) |
| --- | --- |
| /Users/moi/.local/bin/gclaude | dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e |
| /Users/moi/.claude-glm/settings.json | ca6b9b4b694ce9460e8b4b90dcecb6206559799a2f115592e8f201ab49a85bab |
| /Users/moi/.local/share/claude/versions/2.1.296 | c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937 |
| /Users/moi/.cache/bridget-core/agents.json | c8642f4c08a5b311dde2fa5881c83c42eeb5bcfabc47eb5325a7345e463df539 |

Le registre n'a été que lu (read_to_string). Le test écrit uniquement le
registre privé dans la racine de fixture. Hash registre consigné après
exécution ; pas de hash avant capturé séparément, lecture seule par
construction.

## 7. Résidus et production

- Aucun processus résiduel du test (cible worktree et relay.py : aucun).
- Aucun processus tué. Aucun SIGTERM nécessaire : nettoyage spontané
  confirmé par le test (EOF/stop explicites).
- Racines /tmp/nd148-* : aucune (tests secs nettoyés).
- /tmp/ng148-bfcc36f958 : preuve R2 conservée (0700, fichiers 0600).
- /tmp/ng148-8a8f559e77 : vestige R1 (06:48, sans reçu) conservé tel quel.
- Processus production bridget mcp : tous vivants avant/après, non touchés.

## 8. Conclusions

Identité T3 corrigée : aucun T3 dans le chemin natif enfant. Délégation
native en un appel autonome : un seul bridget_delegate, une tâche, un
enfant, une réponse corrélée. Modèle exact glm-5.3 demandé et annoncé.
Résultat durable : ligne unique native_delegations avec modèle et effort
null (catalogue sans efforts). Sources d'auth/settings/CLI intactes.
Le dépôt n'a pas été modifié par le testeur.
