# infra/openbao/policies/rs-worker.hcl: policy identitas OpenBao proses `rs-backend worker` (ADR-025 butir 7,
# ADR-030 butir 1 dan langkah 3, KEPUTUSAN-79, KEPUTUSAN-305).
#
# Hanya baca rahasia klien Keycloak `manage-users` worker IdP per realm (KV v2 mount `secret`, field
# `client_secret`; path = IDP_WORKER_KV_PATH_<REALM>). Tidak ada hak tulis, daftar, hapus, atau metadata:
# rotasi rahasia dilakukan operator lewat skrip deploy, bukan proses ini.
#
# Langkah ini hanya realm `korporat` (CR-026). Realm `pasien` dan `admin` ditambahkan saat alurnya dipindahkan ke
# worker (ADR-030 "Perubahan implementasi" langkah 5-6), dengan path yang sama polanya.
#
# Policy `rs-api.hcl` TIDAK boleh memuat path ini (diuji apps/rs-backend/tests/identity_idp_openbao_policy.rs).
# Penerapan ke OpenBao staging/production lewat skrip deploy bastion (ADR-021, ADR-025); belum diotomatiskan di
# repo ini `[TIDAK DIKETAHUI]` nama metode auth identitas proses (AppRole/token per host, KEPUTUSAN-79).

path "secret/data/rs/keycloak/korporat/idp-worker" {
  capabilities = ["read"]
}
