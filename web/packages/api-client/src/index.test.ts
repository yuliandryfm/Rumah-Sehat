import { describe, expect, test } from "vitest";
import { createOpsClient } from "./index.ts";

describe("createOpsClient", () => {
  test("GET /health/live → memanggil URL dasar + path dan mengembalikan data", async () => {
    const dipanggil: string[] = [];
    const client = createOpsClient({
      baseUrl: "http://api.test",
      fetch: (request: Request) => {
        dipanggil.push(`${request.method} ${request.url}`);
        return Promise.resolve(
          new Response(JSON.stringify({ status: "live" }), {
            status: 200,
            headers: { "content-type": "application/json" },
          }),
        );
      },
    });
    const { data, error, response } = await client.GET("/health/live");
    expect(dipanggil).toEqual(["GET http://api.test/health/live"]);
    expect(response.status).toBe(200);
    expect(error).toBeUndefined();
    expect(data).toBeDefined();
  });

  test("GET /health/ready dengan 503 → mengembalikan error, bukan data", async () => {
    const client = createOpsClient({
      baseUrl: "http://api.test",
      fetch: () =>
        Promise.resolve(
          new Response(JSON.stringify({ correlation_id: "x", response_code: "5030001" }), {
            status: 503,
            headers: { "content-type": "application/json" },
          }),
        ),
    });
    const { data, error } = await client.GET("/health/ready");
    expect(data).toBeUndefined();
    expect(error).toBeDefined();
  });
});
