import { ndJsonStream, type AgentApp, type AgentConnection } from "@agentclientprotocol/sdk";
import { Readable, Writable } from "node:stream";

/** Serve an ACP agent over newline-delimited JSON on stdin and stdout. */
export function serveStdio(app: AgentApp): AgentConnection {
  const stream = ndJsonStream(
    Writable.toWeb(process.stdout) as WritableStream<Uint8Array>,
    Readable.toWeb(process.stdin) as ReadableStream<Uint8Array>,
  );
  return app.connect(stream);
}
