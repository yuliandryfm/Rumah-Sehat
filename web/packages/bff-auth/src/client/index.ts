// @rs/bff-auth/client: komponen klien BFF. Tidak boleh mengimpor modul server paket ini (dipastikan test
// `client/batas-klien.test.ts`): browser hanya memegang cookie sesi dan token CSRF, tidak pernah token OIDC.

export { SessionTimeoutWarning, type SessionTimeoutWarningProps } from "./SessionTimeoutWarning.tsx";
