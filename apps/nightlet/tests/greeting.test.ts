import { client, methods, PROTOCOL_VERSION } from "@agentclientprotocol/sdk";
import type { Message } from "@earendil-works/pi-ai";
import { fauxAssistantMessage, fauxProvider } from "@earendil-works/pi-ai";
import { createModels } from "@earendil-works/pi-ai/models";
import { describe, expect, it } from "vitest";
import { createNightlet } from "../src/nightlet.ts";

const MODEL_ID = "greeting-test";

function setup(): {
  faux: ReturnType<typeof fauxProvider>;
  models: ReturnType<typeof createModels>;
  model: { provider: string; modelId: string };
} {
  const faux = fauxProvider({ provider: "faux", models: [{ id: MODEL_ID }] });
  const models = createModels();
  models.setProvider(faux.provider);
  return { faux, models, model: { provider: faux.provider.id, modelId: MODEL_ID } };
}

describe("nightlet over ACP", () => {
  it("initializes and answers a prompt with the greeting agent", async () => {
    const { faux, models, model } = setup();
    const nightlet = await createNightlet({ models, model });
    let seen: readonly Message[] | undefined;
    faux.setResponses([
      (context) => {
        seen = context.messages;
        return fauxAssistantMessage("Hello, I am Nightlet!");
      },
    ]);

    try {
      const clientApp = client({ name: "nightlet-test" });
      await clientApp.connectWith(nightlet.app, async (context) => {
        const initialized = await context.request(methods.agent.initialize, {
          protocolVersion: PROTOCOL_VERSION,
          clientCapabilities: {},
        });
        expect(initialized.protocolVersion).toBe(PROTOCOL_VERSION);
        expect(initialized.agentInfo).toMatchObject({ name: "nightlet" });

        const session = await context.buildSession(process.cwd()).start();
        try {
          const pending = session.prompt("Hi there!");
          const text = await session.readText();
          const response = await pending;

          expect(text).toBe("Hello, I am Nightlet!");
          expect(response.stopReason).toBe("end_turn");
        } finally {
          session.dispose();
        }
      });
    } finally {
      await nightlet.close();
    }

    expect(seen?.[0]?.role).toBe("system");
    expect(JSON.stringify(seen)).toContain("Nightlet");
  });

  it("keeps a durable transcript across prompts in one session", async () => {
    const { faux, models, model } = setup();
    const nightlet = await createNightlet({ models, model });
    let secondRequest: readonly Message[] | undefined;
    faux.setResponses([
      fauxAssistantMessage("Hello, Alice!"),
      (context) => {
        secondRequest = context.messages;
        return fauxAssistantMessage("You told me your name is Alice.");
      },
    ]);

    try {
      const clientApp = client({ name: "nightlet-test" });
      await clientApp.connectWith(nightlet.app, async (context) => {
        await context.request(methods.agent.initialize, {
          protocolVersion: PROTOCOL_VERSION,
          clientCapabilities: {},
        });
        const session = await context.buildSession(process.cwd()).start();
        try {
          const first = session.prompt("My name is Alice.");
          expect(await session.readText()).toBe("Hello, Alice!");
          await first;

          const second = session.prompt("What is my name?");
          expect(await session.readText()).toBe("You told me your name is Alice.");
          await second;
        } finally {
          session.dispose();
        }
      });
    } finally {
      await nightlet.close();
    }

    const transcript = JSON.stringify(secondRequest);
    expect(transcript).toContain("My name is Alice.");
    expect(transcript).toContain("Hello, Alice!");
  });

  it("reports an unknown session as a resource-not-found error", async () => {
    const { models, model } = setup();
    const nightlet = await createNightlet({ models, model });

    try {
      const clientApp = client({ name: "nightlet-test" });
      await clientApp.connectWith(nightlet.app, async (context) => {
        await context.request(methods.agent.initialize, {
          protocolVersion: PROTOCOL_VERSION,
          clientCapabilities: {},
        });
        await expect(
          context.request(methods.agent.session.prompt, {
            sessionId: "unknown-session",
            prompt: [{ type: "text", text: "Hello?" }],
          }),
        ).rejects.toMatchObject({ code: -32002 });
      });
    } finally {
      await nightlet.close();
    }
  });
});
