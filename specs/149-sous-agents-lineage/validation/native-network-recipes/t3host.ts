// Recette native 149 - hôte MCP T3 réel (T036/T039).
// Couches RÉELLES : serveur HTTP Node, registre d'authentification de sessions MCP,
// toolkit/handlers/OrchestratorMcpService (bridget_session v1/v2), magasin de faits
// de permissions du fournisseur (@t3tools/provider-core).
// Couches SIMULÉES (nommées) : projection de conversation (runs actifs), adaptateur
// fournisseur (le fait de politique est publié par la recette via l'API publique
// publishMcpProviderPermissions, comme le ferait l'adaptateur juste avant turn/start).
// Aucun fournisseur, aucun modèle, aucune base T3 : la projection est en mémoire.
// Même gabarit que BridgetRustInterop.test.ts (R4) ; aucun fichier T3 modifié.
import * as NodeHttp from "node:http";
import { NodeHttpServer } from "@effect/platform-node";
import * as NodeServices from "@effect/platform-node/NodeServices";
import {
  EnvironmentId,
  ProviderInstanceId,
  ThreadId,
  type OrchestrationV2ThreadProjection,
} from "@t3tools/contracts";
import * as McpProviderSession from "@t3tools/provider-core/server/mcpSession";
import * as Context from "effect/Context";
import * as Effect from "effect/Effect";
import * as Exit from "effect/Exit";
import * as Layer from "effect/Layer";
import * as Scope from "effect/Scope";
import { HttpRouter } from "effect/http";
import * as ProviderAdapterRegistry from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/ProviderAdapterRegistry.ts";
import * as ThreadManagement from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/ThreadManagementService.ts";
import * as ProviderRegistry from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/provider/ProviderRegistry.ts";
import * as ProjectService from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/project/ProjectService.ts";
import * as ScheduledTaskService from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/scheduledTasks/ScheduledTaskService.ts";
import * as SecretRequests from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/secrets/SecretRequests.ts";
import * as ServerEnvironment from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/environment/ServerEnvironment.ts";
import * as McpHttpServer from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/McpHttpServer.ts";
import * as McpSessionRegistry from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/McpSessionRegistry.ts";
import * as OrchestratorService from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/OrchestratorMcpService.ts";
import * as ThreadMetadataMcpService from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/ThreadMetadataMcpService.ts";
import * as OrchestratorHandlers from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/toolkits/orchestrator/handlers.ts";
import { OrchestratorToolkit } from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/toolkits/orchestrator/tools.ts";
import {
  idleThreadProjection,
  liveThreadShell,
} from "/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/McpToolAccess.testkit.ts";

export const ENVIRONMENT = "environment:native-recipe149";
const PROVIDER = "codex";

export type SeenRequest = { method: string; session: string | undefined; status: number };

export type Fact = Record<string, unknown>;

export interface T3Host {
  readonly port: number;
  readonly seen: SeenRequest[];
  /** Fil -> runs actifs (projection simulée, en mémoire). */
  readonly runs: Map<string, Array<Record<string, unknown>>>;
  /** Émet un credential MCP réel pour le fil et monte le credential côté fournisseur. */
  issue(threadId: string): Promise<{ config: any }>;
  /** Révocation réelle du registre (arrêt de session / fil). */
  revokeThread(threadId: string): Promise<void>;
  /** Rotation réelle : révocation puis nouvelle émission, comme issueActiveMcpCredential. */
  rotate(threadId: string): Promise<{ config: any }>;
  /** Démarre un tour actif simulé pour le fil et retourne ses identifiants. */
  startRun(threadId: string, runId: string): { runId: string; attemptId: string; providerThreadId: string };
  /** Fin du tour : l'adaptateur retire le fait du run propriétaire (tombstone). */
  endRun(threadId: string, config: any, runId: string): void;
  /** Publication du fait de permissions par l'« adaptateur » (API publique réelle). */
  publish(config: any, fact: Fact, binding: { attemptId: string; providerThreadId: string }): number | undefined;
  /** Ferme réellement le serveur HTTP et le registre (panne ou arrêt de T3). */
  stop(): Promise<void>;
}

export const codexFullAccessFact = (
  config: any,
  runId: string,
  cwd: string,
  over: Fact = {},
): Fact => ({
  version: 1,
  source: "provider_turn",
  run_id: runId,
  provider_session_id: config.providerSessionId,
  provider_instance_id: config.providerInstanceId,
  driver: "codex_app_server",
  cwd,
  runtime_mode: "full-access",
  interaction_mode: "default",
  provider_policy: {
    kind: "codex",
    approval_policy: "never",
    approvals_reviewer: "user",
    sandbox_policy: { type: "dangerFullAccess" },
  },
  ...over,
});

export async function startT3Host(port: number): Promise<T3Host> {
  const environmentId = EnvironmentId.make(ENVIRONMENT);
  const providerInstanceId = ProviderInstanceId.make(PROVIDER);
  const seen: SeenRequest[] = [];
  const runs = new Map<string, Array<Record<string, unknown>>>();
  const projectionOf = (id: ThreadId) => {
    const base = idleThreadProjection(liveThreadShell(id));
    return { ...base, runs: runs.get(id) ?? [] } as unknown as OrchestrationV2ThreadProjection;
  };
  const storageLayer = Layer.mergeAll(
    Layer.mock(ThreadManagement.ThreadManagementService)({
      getThreadShell: (id) => Effect.succeed(liveThreadShell(id)),
      getThreadRecords: (id) => Effect.succeed(projectionOf(id)),
    }),
    Layer.mock(ProviderRegistry.ProviderRegistry)({ getProviders: Effect.succeed([]) }),
    Layer.mock(ProviderAdapterRegistry.ProviderAdapterRegistryV2)({
      list: () => Effect.succeed([]),
    }),
    Layer.mock(ProjectService.ProjectService)({}),
    Layer.mock(SecretRequests.SecretRequests)({}),
    Layer.mock(ScheduledTaskService.ScheduledTaskService)({}),
    Layer.mock(ThreadMetadataMcpService.ThreadMetadataMcpService)({}),
    NodeServices.layer,
  );
  const registryLayer = McpSessionRegistry.layer.pipe(
    Layer.provide(
      Layer.mock(ServerEnvironment.ServerEnvironment)({
        getEnvironmentId: Effect.succeed(environmentId),
      }),
    ),
    Layer.provide(NodeServices.layer),
  );
  const httpLayer = NodeHttpServer.layer(
    () => {
      const server = NodeHttp.createServer();
      // Observation passive : méthode, présence de session de transport, statut.
      // Jamais d'en-tête d'autorisation ni de corps.
      server.on("request", (request, response) => {
        const session = request.headers["mcp-session-id"];
        response.on("finish", () =>
          seen.push({
            method: request.method!,
            session: typeof session === "string" ? "present" : undefined,
            status: response.statusCode,
          }),
        );
      });
      return server;
    },
    { port, host: "127.0.0.1" },
  );
  // Un seul cycle de vie : le registre fige l'endpoint du serveur HTTP au build, et
  // un redémarrage de T3 perd ses credentials (registre en mémoire), comme en production.
  const scope = await Effect.runPromise(Scope.make());
  const httpContext = await Effect.runPromise(Layer.buildWithScope(httpLayer, scope));
  const registryContext = await Effect.runPromise(
    Layer.buildWithScope(
      registryLayer.pipe(Layer.provide(Layer.succeedContext(httpContext))),
      scope,
    ),
  );
  const registry = Context.get(registryContext, McpSessionRegistry.McpSessionRegistry);
  const serverLayer = McpHttpServer.toolkitRegistration(
    OrchestratorToolkit,
    OrchestratorHandlers.layer,
  ).pipe(
    Layer.provideMerge(McpHttpServer.layerMcpTransport),
    Layer.provide(OrchestratorService.layer.pipe(Layer.provideMerge(storageLayer))),
  );
  const fullContext = Context.merge(httpContext, registryContext);
  await Effect.runPromise(
    Layer.buildWithScope(
      HttpRouter.serve(serverLayer, { disableListenLog: true, disableLogger: true }).pipe(
        Layer.provide(Layer.succeedContext(fullContext)),
      ),
      scope,
    ),
  );

  const issue = async (threadId: string) => {
    const issued = await Effect.runPromise(
      registry.issue({ threadId: ThreadId.make(threadId), providerInstanceId }),
    );
    // L'adaptateur monte le credential exact qu'il a reçu du registre.
    McpProviderSession.setMcpProviderSession(issued.config as any);
    return issued as { config: any };
  };
  return {
    port,
    seen,
    runs,
    issue,
    revokeThread: async (threadId) => {
      await Effect.runPromise(registry.revokeThread(ThreadId.make(threadId)));
    },
    rotate: async (threadId) => {
      await Effect.runPromise(registry.revokeThread(ThreadId.make(threadId)));
      return issue(threadId);
    },
    startRun: (threadId, runId) => {
      const binding = { attemptId: `attempt-${runId}`, providerThreadId: `native-${runId}` };
      runs.set(threadId, [
        {
          id: runId,
          ordinal: 1,
          status: "running",
          providerInstanceId,
          activeAttemptId: binding.attemptId,
          providerThreadId: binding.providerThreadId,
        },
      ]);
      return { runId, ...binding };
    },
    endRun: (threadId, config, runId) => {
      // Tombstone du run propriétaire (fin/échec/close) : la projection garde le run vivant,
      // comme dans les tests T3, pour prouver qu'on ne retombe jamais sur l'enveloppe v1.
      McpProviderSession.invalidateMcpProviderPermissions(config, runId);
    },
    publish: (config, fact, binding) =>
      McpProviderSession.publishMcpProviderPermissions(config, fact as any, {
        attemptId: binding.attemptId,
        providerThreadId: binding.providerThreadId,
      }),
    stop: async () => {
      await Effect.runPromise(Scope.close(scope, Exit.void));
    },
  };
}
