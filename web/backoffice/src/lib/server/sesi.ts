import { cookieNames } from "@rs/bff-auth";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { BFF_APP, getCore } from "./bff.ts";

/**
 * Penjaga halaman terautentikasi (server component). Tanpa cookie sesi: langsung ke login tanpa menyentuh
 * Valkey. Dengan cookie: sesi dimuat dan batas tidak aktif/mutlak ditegakkan oleh `SessionManager`.
 * Kegagalan Valkey/OpenBao dilempar (jatuh ke `error.tsx` dengan kode referensi), bukan dianggap "belum masuk".
 * Mengembalikan token CSRF sesi untuk formulir keluar. Bukan otorisasi data: itu tetap di API Rust (ADR-012).
 */
export async function wajibSesi(): Promise<{ readonly csrfToken: string }> {
  const nilai = (await cookies()).get(cookieNames(BFF_APP).session)?.value;
  if (nilai === undefined) {
    redirect("/auth/login");
  }
  const core = await getCore();
  const hasil = await core.sessions.load(nilai);
  if (hasil.status !== "active") {
    redirect("/auth/login");
  }
  return { csrfToken: hasil.session.csrfToken };
}
