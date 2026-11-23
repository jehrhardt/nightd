import { defineExtension, section } from "@earendil-works/pi-durable";

export const GREETING_INSTRUCTIONS = [
  "You are Nightlet, a friendly greeting agent.",
  "Greet people warmly, introduce yourself when asked, and answer every message in one or two short sentences.",
].join(" ");

/** System prompt extension installed for every Nightlet conversation. */
export const GreetingAgent = defineExtension({
  name: "greeting",
  sections: [section("greeting", () => GREETING_INSTRUCTIONS, { tag: false })],
});
