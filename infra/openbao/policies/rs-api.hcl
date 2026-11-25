# infra/openbao/policies/rs-api.hcl: policy identitas OpenBao proses `rs-backend api` (ADR-025 butir 7,
# KEPUTUSAN-79, KEPUTUSAN-293, KEPUTUSAN-305).
#
# Isi = kebutuhan `ApiConfig` saat ini (apps/rs-backend/src/config.rs `IdentityKeysConfig`, nilai bawaan):
# - Transit `transit`, kunci `rs-dek-kek`: `datakey/plaintext` (DEK baru) dan `decrypt` (membuka DEK tersimpan),
#   crates/crypto/src/transit.rs.
# - KV v2 `secret`, kunci blind index NIK (NIK_BLIND_INDEX_KV_PATH; contoh lokal `ruang-sehat/blind-index/nik`),
#   crates/crypto/src/blind_index.rs.
# Path berbeda per lingkungan disesuaikan skrip deploy; prinsipnya tetap: daftar izin eksplisit, tanpa wildcard.
#
# SENGAJA TIDAK ADA: rahasia klien Keycloak `manage-users` (`secret/data/rs/keycloak/*/idp-worker`, hanya
# `rs-worker.hcl`; ADR-030 butir 1). Tanpa izin, OpenBao menolak (deny bawaan); diuji
# apps/rs-backend/tests/identity_idp_openbao_policy.rs.

path "transit/datakey/plaintext/rs-dek-kek" {
  capabilities = ["update"]
}

path "transit/decrypt/rs-dek-kek" {
  capabilities = ["update"]
}

path "secret/data/ruang-sehat/blind-index/nik" {
  capabilities = ["read"]
}
