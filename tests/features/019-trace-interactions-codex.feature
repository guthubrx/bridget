# language: fr
Fonctionnalité: Expliquer un tour Codex suspendu
  Afin de diagnostiquer une panne sans inventer sa cause
  En tant qu'opérateur Bridget
  Je veux conserver la requête fournisseur qui suspend un tour

  Scénario: Une requête bloquante précède une échéance
    Étant donné un pilote qui accepte un tour
    Et qui émet une requête serveur vers le client contenant une sentinelle sensible
    Quand le pilote reste silencieux jusqu'à l'échéance
    Alors la requête pendante est journalisée avant l'erreur
    Et la requête et l'erreur désignent le même message et le même tour
    Et la sentinelle sensible est absente du journal

  Plan du scénario: Une chaîne libre ne traverse pas un champ autorisé
    Étant donné un pilote qui accepte un tour
    Et qui injecte une sentinelle dans <champ>
    Quand Bridget journalise la requête puis sa borne terminale
    Alors la sentinelle est attestée à l'entrée
    Et les deux événements et leurs corrélations restent présents
    Et la sentinelle est absente du journal

    Exemples:
      | champ                         |
      | méthode avec ESC, CR et bidi |
      | identifiant JSON-RPC chaîne  |
      | identifiant de tour          |
      | raison avec ESC, CR et bidi  |
