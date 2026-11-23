import {
  agent,
  methods,
  PROTOCOL_VERSION,
  RequestError,
  type AgentApp,
  type ContentBlock,
  type StopReason,
} from "@agentclientprotocol/sdk";
import { BACKGROUND_CONTEXT, withAbortSignal } from "@earendil-works/chord/context";
import type { AssistantMessage, Message } from "@earendil-works/pi-ai";
import type { Models } from "@earendil-works/pi-ai/models";
import {
  createRegistry,
  Harness,
  MemoryStorage,
  type Conversation,
  type ModelRef,
  type Report,
  type Storage,
} from "@earendil-works/pi-durable";
import { DEFAULT_MODEL, NIGHTLET_VERSION } from "./config.ts";
import { GreetingAgent } from "./greeting.ts";

interface SessionState {
  readonly conversation: Conversation;
  readonly aborts: Set<AbortController>;
}

export interface CreateNightletOptions {
  /** Model access used by the durable harness. */
  readonly models: Models;
  /** Durable storage; in-memory by default. */
  readonly storage?: Storage;
  /** Model that answers prompts. */
  readonly model?: ModelRef;
  /** Receives non-fatal harness failures, for example a failed task. */
  readonly onReport?: (report: Report) => void;
}

export interface Nightlet {
  readonly app: AgentApp;
  readonly harness: Harness;
  close(): Promise<void>;
}

/**
 * Build the ACP agent around a Pi Durable harness and a greeting agent.
 *
 * The caller owns the returned harness and must close it, which `close()` does.
 */
export async function createNightlet(options: CreateNightletOptions): Promise<Nightlet> {
  const registry = createRegistry();
  registry.install(GreetingAgent);

  const model: ModelRef = options.model ?? DEFAULT_MODEL;
  const harness = await Harness.open(
    options.storage ?? new MemoryStorage(),
    { models: options.models, registry, onReport: options.onReport },
    BACKGROUND_CONTEXT,
  );

  const sessions = new Map<string, SessionState>();

  const app = agent({ name: "nightlet" })
    .onRequest(methods.agent.initialize, ({ params }) => ({
      protocolVersion: PROTOCOL_VERSION,
      agentCapabilities: { promptCapabilities: {} },
      agentInfo: { name: "nightlet", version: NIGHTLET_VERSION },
      _meta: { requestedProtocolVersion: params.protocolVersion },
    }))
    .onRequest(methods.agent.session.new, async ({ params }) => {
      const conversation = await harness.createConversation(
        {
          ownership: { kind: "ownerless" },
          agent: { model, cwd: params.cwd, extensions: [GreetingAgent] },
        },
        BACKGROUND_CONTEXT,
      );
      const sessionId = String(conversation.id);
      sessions.set(sessionId, { conversation, aborts: new Set() });
      return { sessionId };
    })
    .onRequest(methods.agent.session.prompt, async ({ params, client, signal }) => {
      const session = sessions.get(params.sessionId);
      if (session === undefined) throw RequestError.sessionNotFound(params.sessionId);

      const text = promptText(params.prompt);
      if (text.length === 0) return { stopReason: "end_turn" satisfies StopReason };

      const abort = new AbortController();
      const cancel = () => abort.abort();
      signal.addEventListener("abort", cancel);
      session.aborts.add(abort);
      const context = withAbortSignal(abort.signal, BACKGROUND_CONTEXT);

      try {
        const submission = await session.conversation.submit(
          { type: "input", content: text },
          context,
        );
        const settled = await submission.wait(context);

        if (settled.status !== "done" || settled.type !== "input") {
          if (settled.status === "unanswered" && settled.reason === "aborted") {
            return { stopReason: "cancelled" satisfies StopReason };
          }
          if (settled.status === "unanswered" && settled.reason === "turn_limit") {
            return { stopReason: "max_turn_requests" satisfies StopReason };
          }
          const reason = settled.status === "unanswered" ? settled.reason : settled.status;
          const detail =
            settled.status === "unanswered" && settled.detail !== undefined
              ? `: ${JSON.stringify(settled.detail)}`
              : "";
          throw new Error(`nightlet could not answer (${reason})${detail}`);
        }

        const entry = await session.conversation.commit(
          (tx) => tx.entry(settled.answer),
          BACKGROUND_CONTEXT,
        );
        const answer = assistantMessage(entry?.model);
        const reply = messageText(answer);
        if (reply.length > 0) {
          await client.notify(methods.client.session.update, {
            sessionId: params.sessionId,
            update: {
              sessionUpdate: "agent_message_chunk",
              content: { type: "text", text: reply },
            },
          });
        }
        return { stopReason: acpStopReason(answer) };
      } catch (error) {
        if (abort.signal.aborted) {
          await session.conversation.abort(BACKGROUND_CONTEXT).catch(() => undefined);
          return { stopReason: "cancelled" satisfies StopReason };
        }
        throw error;
      } finally {
        signal.removeEventListener("abort", cancel);
        session.aborts.delete(abort);
      }
    })
    .onNotification(methods.agent.session.cancel, ({ params }) => {
      const session = sessions.get(params.sessionId);
      if (session === undefined) return;
      for (const abort of session.aborts) abort.abort();
    });

  return {
    app,
    harness,
    close: () => harness.close(BACKGROUND_CONTEXT),
  };
}

function promptText(blocks: readonly ContentBlock[]): string {
  const parts: string[] = [];
  for (const block of blocks) {
    if (block.type === "text") parts.push(block.text);
  }
  return parts.join("\n").trim();
}

function assistantMessage(messages: readonly Message[] | undefined): AssistantMessage | undefined {
  const first = messages?.[0];
  return first?.role === "assistant" ? first : undefined;
}

function messageText(message: AssistantMessage | undefined): string {
  const parts: string[] = [];
  for (const block of message?.content ?? []) {
    if (block.type === "text") parts.push(block.text);
  }
  return parts.join("");
}

function acpStopReason(message: AssistantMessage | undefined): StopReason {
  switch (message?.stopReason) {
    case "length":
      return "max_tokens";
    case "aborted":
      return "cancelled";
    case "error":
      return "refusal";
    default:
      return "end_turn";
  }
}
