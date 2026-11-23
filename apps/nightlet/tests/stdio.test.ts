import { client, methods, ndJsonStream, PROTOCOL_VERSION } from "@agentclientprotocol/sdk";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { Readable, Writable } from "node:stream";
import { describe, expect, it } from "vitest";

const FIXTURE = new URL("./fixtures/stdio-agent.ts", import.meta.url);

describe("nightlet stdio transport", () => {
  it("speaks ACP over the stdin/stdout of a child process", async () => {
    const child = spawn(process.execPath, [FIXTURE.pathname], {
      stdio: ["pipe", "pipe", "inherit"],
      env: { ...process.env, NIGHTLET_TEST_REPLY: "Hello over stdio!" },
    });

    const stdin = child.stdin;
    const stdout = child.stdout;
    if (stdin === null || stdout === null) throw new Error("child process has no stdio");

    try {
      const stream = ndJsonStream(
        Writable.toWeb(stdin) as WritableStream<Uint8Array>,
        Readable.toWeb(stdout) as ReadableStream<Uint8Array>,
      );
      const clientApp = client({ name: "nightlet-stdio-test" });
      await clientApp.connectWith(stream, async (context) => {
        const initialized = await context.request(methods.agent.initialize, {
          protocolVersion: PROTOCOL_VERSION,
          clientCapabilities: {},
        });
        expect(initialized.agentInfo).toMatchObject({ name: "nightlet" });

        const session = await context.buildSession(process.cwd()).start();
        try {
          const pending = session.prompt("Hi from the test client");
          const text = await session.readText();
          const response = await pending;
          expect(text).toBe("Hello over stdio!");
          expect(response.stopReason).toBe("end_turn");
        } finally {
          session.dispose();
        }
      });
    } finally {
      if (child.exitCode === null) {
        child.kill();
        await once(child, "exit");
      }
    }
  });
});
