import { fauxAssistantMessage, fauxProvider } from "@earendil-works/pi-ai";
import { createModels } from "@earendil-works/pi-ai/models";
import { createNightlet } from "../../src/nightlet.ts";
import { serveStdio } from "../../src/stdio.ts";

const faux = fauxProvider({ provider: "faux", models: [{ id: "greeting-test" }] });
faux.setResponses([
  fauxAssistantMessage(process.env.NIGHTLET_TEST_REPLY ?? "Hello from Nightlet!"),
]);

const models = createModels();
models.setProvider(faux.provider);

const nightlet = await createNightlet({
  models,
  model: { provider: faux.provider.id, modelId: "greeting-test" },
});

const connection = serveStdio(nightlet.app);
await connection.closed;
await nightlet.close();
