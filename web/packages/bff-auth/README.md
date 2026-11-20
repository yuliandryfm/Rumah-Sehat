# @rs/bff-auth

BFF autentikasi untuk server Next.js (ADR-011 butir 7, ADR-024 butir 3, CR-012). **Modul kritis T1**: setiap perubahan wajib review penuh (code-reviewer + security) dan test negatif (`docs/coding-standards.md` §7.3).

Hanya untuk kode server. Komponen klien tidak boleh mengimpor paket ini.

## Isi (F1-T41)

| Berkas | Isi |
| --- | --- |
| `src/config.ts` | Konfigurasi bertipe dari variabel `BFF_*`, `OPENBAO_*` (gagal cepat, tanpa mencetak nilai) |
| `src/openbao.ts` | Rahasia BFF dari OpenBao KV v2 `[SRC-488]` |
| `src/token-crypto.ts` | JWE `dir`/`A256GCM` (jose 6.2.17) dengan kunci per sesi HKDF-SHA256 dari kunci induk OpenBao dan ID sesi mentah `[SRC-590]` |
| `src/store.ts` | Penyimpanan sesi di Valkey lewat ioredis 6.0.0 `[SRC-591]` |
| `src/session.ts` | Siklus hidup sesi: tidak aktif 30 menit, mutlak 12 jam (ADR-011 butir 7, NFR-SEC-04), transaksi login sekali pakai 10 menit, kunci penyegaran |
| `src/oidc.ts` | Authorization Code + PKCE S256 klien rahasia (openid-client 6.9.0) `[SRC-592]`; logout Keycloak dari server `[SRC-593]` |
| `src/core.ts` | Perakitan: konfigurasi → rahasia → Valkey → JWE → sesi → OIDC |

## Rahasia di OpenBao KV v2

Path: `<BFF_OPENBAO_KV_MOUNT>/data/<BFF_OPENBAO_KV_PATH>`, misalnya `secret/data/rs/bff/backoffice`. Identitas proses BFF hanya punya `read` pada path itu (KEPUTUSAN-79).

| Field | Isi |
| --- | --- |
| `oidc_client_secret` | Secret klien Keycloak (`rs-backoffice-bff`) |
| `session_key` | Kunci induk enkripsi token sesi: 32 byte acak, base64url tanpa padding |
| `session_key_id` | ID kunci `[a-z0-9-]{1,32}` (header `kid`) |
| `session_key_previous`, `session_key_previous_id` | Opsional saat rotasi: kunci lama hanya untuk membuka sesi yang masih aktif |
| `valkey_password` | Password Valkey (`requirepass`) |

Contoh lokal (OpenBao mode dev compose, nilai sintetis):

```sh
docker compose -f infra/local/compose.yaml exec -e BAO_TOKEN="$BAO_DEV_ROOT_TOKEN" openbao \
  bao kv put secret/rs/bff/backoffice oidc_client_secret="$RS_KC_BACKOFFICE_BFF_SECRET" \
  valkey_password="$VALKEY_APP_PASSWORD" session_key_id=k1 \
  session_key="$(node -e 'process.stdout.write(require("crypto").randomBytes(32).toString("base64url"))')"
```

## Data di Valkey

`rs:bff:<app>:s:<sha256(ID sesi)>` (HASH: waktu dibuat, aktivitas terakhir, batas mutlak, token CSRF, token JWE), `rs:bff:<app>:l:<sha256(ID login)>` (transaksi login JWE, sekali pakai), `rs:bff:<app>:r:<sha256(ID sesi)>` (kunci penyegaran). ID mentah hanya ada di cookie; token hanya tersimpan sebagai JWE yang kuncinya diturunkan dari ID mentah, sehingga pembaca Valkey tidak memperoleh token yang dapat dipakai (M-06).

## Test

- Unit (`*.test.ts`, proyek Vitest `node`): konfigurasi, OpenBao, JWE, sesi (backend memori), OIDC terhadap authorization server tiruan.
- Integrasi (`*.integration.test.ts`, proyek Vitest `integrasi`): Valkey, OpenBao dev, dan Keycloak 26.8.2 dengan realm `admin` dari `infra/keycloak/realms/` di kontainer sekali pakai. **Docker wajib**; tanpa docker test GAGAL, tidak dilewati.

## Belum diketahui / batasan

- TLS ke OpenBao belum diterapkan (sama dengan crate `crypto`): hanya `http://<ip-loopback>` yang diterima.
- Jalur logout Keycloak dari server berstatus "legacy" di kode Keycloak `[SRC-593]`; dipantau saat upgrade Keycloak.
- Rahasia dibaca sekali saat start; pemuatan ulang saat rotasi tanpa restart `[TIDAK DIKETAHUI]` (belum dirancang).
