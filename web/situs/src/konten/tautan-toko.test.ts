import { describe, expect, test } from "vitest";
import { ambilUrlToko } from "./tautan-toko.ts";

const PLAY = "https://play.google.com/store/apps/details?id=id.contoh.uji";
const APPLE = "https://apps.apple.com/id/app/contoh/id0000000000";

describe("ambilUrlToko (CR-038)", () => {
  test("https dengan host persis → href", () => {
    expect(ambilUrlToko("playstore", { SITUS_PLAYSTORE_URL: PLAY })).toBe(PLAY);
    expect(ambilUrlToko("appstore", { SITUS_APPSTORE_URL: APPLE })).toBe(APPLE);
  });

  test("kosong atau tidak diatur → null", () => {
    expect(ambilUrlToko("playstore", {})).toBeNull();
    expect(ambilUrlToko("playstore", { SITUS_PLAYSTORE_URL: "" })).toBeNull();
    expect(ambilUrlToko("appstore", { SITUS_PLAYSTORE_URL: PLAY })).toBeNull();
  });

  test.each([
    ["host lain", "https://evil.test/store/apps/details?id=x"],
    ["host mirip (sufiks)", "https://play.google.com.evil.test/x"],
    ["host mirip (awalan)", "https://evilplay.google.com/x"],
    ["subdomain", "https://www.play.google.com/x"],
    ["tertukar: host App Store untuk Play", "https://apps.apple.com/id/app/x/id1"],
    ["http", "http://play.google.com/store/apps/details?id=x"],
    ["http localhost", "http://localhost:3000"],
    ["userinfo", "https://user:pass@play.google.com/x"],
    ["userinfo tanpa sandi", "https://user@play.google.com/x"],
    ["userinfo mengecoh", "https://play.google.com@evil.test/x"],
    ["port", "https://play.google.com:8443/x"],
    ["host huruf besar", "https://PLAY.GOOGLE.COM/x"],
    ["skema huruf besar", "HTTPS://play.google.com/x"],
    ["javascript", "javascript:alert(1)"],
    ["protocol-relative", "//play.google.com/x"],
    ["relatif", "/store"],
    ["tanpa skema", "play.google.com/x"],
    ["spasi di depan", " https://play.google.com/x"],
    ["spasi di tengah", "https://play.google.com/a b"],
    ["karakter kendali (tab)", "https://play.google.com/x\ty"],
    ["baris baru", "https://play.google.com/x\n"],
    ["NUL", "https://play.google.com/x\u0000"],
    ["karakter kendali C1", "https://play.google.com/x\u0085"],
    ["backslash", "https://play.google.com\\@evil.test/x"],
    ["tanpa host", "https:///x"],
    ["titik di akhir host", "https://play.google.com./x"],
    ["homograf Sirilik", "https://plаy.google.com/x"],
    ["punycode", "https://xn--ply-8cd.google.com/x"],
    ["titik di-encode persen", "https://play%2egoogle.com/x"],
    ["port 443 eksplisit", "https://play.google.com:443/x"],
    ["skema data", "data:text/html,hi"],
    ["skema tanpa garis miring", "https:play.google.com/x"],
  ])("Play: %s → null", (_n, nilai) => {
    expect(ambilUrlToko("playstore", { SITUS_PLAYSTORE_URL: nilai })).toBeNull();
  });

  test.each([
    ["host lain", "https://evil.test/id/app/x/id1"],
    ["tertukar: host Play untuk App Store", PLAY],
    ["http", "http://apps.apple.com/id/app/x/id1"],
    ["userinfo", "https://u:p@apps.apple.com/x"],
    ["host huruf besar", "https://Apps.Apple.com/x"],
    ["host mirip", "https://apps.apple.com.evil.test/x"],
    ["karakter kendali", "https://apps.apple.com/x\r\ny"],
    ["titik di akhir host", "https://apps.apple.com./x"],
    ["homograf Sirilik", "https://apps.аpple.com/x"],
    ["punycode", "https://xn--pple-43d.apple.com/x"],
    ["titik di-encode persen", "https://apps%2eapple.com/x"],
    ["port 443 eksplisit", "https://apps.apple.com:443/x"],
    ["skema data", "data:text/html,hi"],
    ["skema tanpa garis miring", "https:apps.apple.com/x"],
  ])("App Store: %s → null", (_n, nilai) => {
    expect(ambilUrlToko("appstore", { SITUS_APPSTORE_URL: nilai })).toBeNull();
  });
});
