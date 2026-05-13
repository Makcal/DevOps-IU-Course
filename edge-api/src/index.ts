/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Bind resources to your worker in `wrangler.jsonc`. After adding bindings, a type definition for the
 * `Env` object can be regenerated with `npm run cf-typegen`.
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

export interface Env {
  APP_NAME: string;
  COURSE_NAME: string;
  API_TOKEN: string;
  ADMIN_EMAIL: string;
  SETTINGS: KVNamespace;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const method = request.method;

    console.log(`${method} ${url.pathname} - colo: ${request.cf?.colo}`);

    // Health check
    if (url.pathname === "/health" && method === "GET") {
      return Response.json({
        status: "ok",
        timestamp: new Date().toISOString(),
        version: "2.0.0"
      });
    }

    // App info
    if (url.pathname === "/" && method === "GET") {
      return Response.json({
        app: env.APP_NAME,
        course: env.COURSE_NAME,
        message: "Hello from Cloudflare Workers edge!",
        endpoints: ["/health", "/edge", "/counter", "/reset"],
        timestamp: new Date().toISOString()
      });
    }

    // Edge metadata
    if (url.pathname === "/edge" && method === "GET") {
      return Response.json({
        colo: request.cf?.colo || "unknown",
        country: request.cf?.country || "unknown",
        city: request.cf?.city || "unknown",
        continent: request.cf?.continent || "unknown",
        timezone: request.cf?.timezone || "unknown",
        httpProtocol: request.cf?.httpProtocol || "unknown",
        tlsVersion: request.cf?.tlsVersion || "unknown",
        asn: request.cf?.asn || "unknown",
        asOrganization: request.cf?.asOrganization || "unknown"
      });
    }

    // KV-backed counter
    if (url.pathname === "/counter" && method === "GET") {
      const raw = await env.SETTINGS.get("visits");
      const visits = Number(raw ?? "0") + 1;
      await env.SETTINGS.put("visits", String(visits));
      return Response.json({ visits });
    }

    // Reset counter (protected)
    if (url.pathname === "/reset" && method === "POST") {
      const authHeader = request.headers.get("Authorization");
      const token = authHeader?.replace("Bearer ", "");

      if (token !== env.API_TOKEN) {
        return new Response("Unauthorized", { status: 401 });
      }

      await env.SETTINGS.put("visits", "0");
      return Response.json({ message: "Counter reset", visits: 0 });
    }

    return new Response("Not Found", { status: 404 });
  }
} satisfies ExportedHandler<Env>;
