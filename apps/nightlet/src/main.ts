#!/usr/bin/env node
import { BACKGROUND_CONTEXT } from "@earendil-works/chord/context";
import { createModels } from "@earendil-works/pi-ai/models";
import { opencodeGoProvider } from "@earendil-works/pi-ai/providers/opencode-go";
import { openNodeJsonlStorage } from "@earendil-works/pi-durable/storage/jsonl/node";
import { homedir } from "node:os";
import { join } from "node:path";
import { DEFAULT_MODEL } from "./config.ts";
import { createNightlet } from "./nightlet.ts";
import { serveStdio } from "./stdio.ts";

const models = createModels();
models.setProvider(opencodeGoProvider());

const stateHome = process.env.XDG_STATE_HOME ?? join(homedir(), ".local", "state");
const dataDir = process.env.NIGHTLET_DATA_DIR ?? join(stateHome, "nightlet");
const storage = await openNodeJsonlStorage(dataDir, BACKGROUND_CONTEXT);

const nightlet = await createNightlet({
  models,
  storage,
  model: DEFAULT_MODEL,
  onReport: (report) => console.error(`[nightlet] ${report.kind}:`, report.error),
});

const connection = serveStdio(nightlet.app);
await connection.closed;
await nightlet.close();
