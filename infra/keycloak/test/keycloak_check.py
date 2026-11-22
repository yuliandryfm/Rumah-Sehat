"""Test realm `pasien` dan `admin` sebagai kode terhadap Keycloak sungguhan (F1-T31, modul kritis T1).

Dijalankan oleh scripts/keycloak-check.sh setelah konfigurasi diterapkan dengan keycloak-config-cli.
Hanya pustaka standar Python 3. Semua user uji sintetis (domain .invalid) dan dihapus di akhir.

Dua kelompok pemeriksaan:
- konfigurasi lewat Admin REST (nilai realm, alur, required action, klien, profil user);
- perilaku lewat OIDC/halaman login: MFA wajib, kode pemulihan wajib, penguncian 5 gagal, pesan seragam,
  pendaftaran mati, PKCE wajib, rotasi token penyegar, Argon2, kebijakan password, atur ulang password
  tidak melewati OTP (US-103 AC4e), grant password ditolak.
- klien layanan pengelola pengguna admin `rs-backend-admin-manager` (F1-T65): hak minimum manage-users,
  siklus buat → undangan sekali pakai 24 jam → aktivasi (TOTP + password) → nonaktifkan → hapus;
- klien layanan pengambil event `rs-backend-event-reader` (F1-T34): hanya client credentials,
  `realm-management/view-events`, cakupan `kc-admin-api`; bentuk event LOGIN_ERROR yang dipetakan backend.

Jalur email (KC_CHECK_EMAIL_MODE):
- `relay` (mode ephemeral, F1-T36/ADR-026): email realm dikirim apa adanya lewat SMTP realm (STARTTLS) ke relay
  sungguhan, lalu ke tiruan SES yang memvalidasi SigV4; ditambah skenario kegagalan SES (Admin REST 500,
  "lupa password" mencatat EMAIL_SEND_FAILED), STARTTLS dimatikan dan pengirim di luar daftar izin ditolak, serta
  log relay/Keycloak tanpa alamat dan token.
- `sink` (mode compose): SMTP realm sementara diarahkan ke penangkap uji tanpa TLS (fixture), lalu dipulihkan.

Variabel lingkungan wajib: KC_CHECK_BASE_URL, KC_CHECK_ADMIN_USER, KC_CHECK_ADMIN_PASSWORD,
KC_CHECK_SINK_CONTAINER, KC_CHECK_SINK_PORT, KC_CHECK_EMAIL_MODE, dan nilai RS_KC_* yang sama dengan yang dipakai
saat konfigurasi diterapkan; mode relay juga KC_CHECK_RELAY_CONTAINER, KC_CHECK_KC_CONTAINER,
KC_CHECK_SES_CONFIG_SET. Nilai rahasia tidak pernah dicetak.
"""

import difflib
import email
import email.header
import html
import json
import os
import re
import secrets
import subprocess
import sys
import time
import urllib.parse

sys.dont_write_bytecode = True  # tanpa __pycache__ di repo
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kc_http as k  # noqa: E402

PESAN_SERAGAM = (
    "Email, password, atau kode tidak sesuai. Bila tidak dapat masuk, gunakan Lupa password atau hubungi "
    "Layanan Pelanggan."
)
JEDA = 1.2  # > Quick Login Check 1000 ms (KEPUTUSAN-190): percobaan lebih cepat memicu kunci 1 menit


def env(name):
    v = os.environ.get(name)
    if not v:
        print(f"variabel lingkungan wajib tidak ada: {name}", file=sys.stderr)
        sys.exit(2)
    return v


BASE = env("KC_CHECK_BASE_URL").rstrip("/")
ADMIN_USER = env("KC_CHECK_ADMIN_USER")
ADMIN_PASSWORD = env("KC_CHECK_ADMIN_PASSWORD")
SINK = env("KC_CHECK_SINK_CONTAINER")
SINK_PORT = env("KC_CHECK_SINK_PORT")
EMAIL_MODE = env("KC_CHECK_EMAIL_MODE")
if EMAIL_MODE not in ("relay", "sink"):
    print("KC_CHECK_EMAIL_MODE harus relay atau sink", file=sys.stderr)
    sys.exit(2)
RELAY = env("KC_CHECK_RELAY_CONTAINER") if EMAIL_MODE == "relay" else None
KC_CONTAINER = env("KC_CHECK_KC_CONTAINER") if EMAIL_MODE == "relay" else None
SES_CONFIG_SET = env("KC_CHECK_SES_CONFIG_SET") if EMAIL_MODE == "relay" else None
RINCI = os.environ.get("KC_CHECK_RINCI") == "1"

KLIEN = {
    "pasien": {
        "client_id": "rs-pasien-mobile",
        "redirect": env("RS_KC_PASIEN_MOBILE_REDIRECT_URI"),
        "secret": None,
    },
    "admin": {
        "client_id": "rs-backoffice-bff",
        "redirect": env("RS_KC_BACKOFFICE_BFF_REDIRECT_URI"),
        "secret": env("RS_KC_BACKOFFICE_BFF_SECRET"),
    },
}
KLIEN_LAIN = {
    "admin": [("rs-portal-bff", env("RS_KC_PORTAL_BFF_REDIRECT_URI"), False)],
    "pasien": [],
}
SESI = {"pasien": (2592000, 7776000), "admin": (1800, 43200)}

LULUS = 0
GAGAL = 0


def periksa(kondisi, pesan, rincian=""):
    global LULUS, GAGAL
    if kondisi:
        LULUS += 1
        if RINCI:
            print(f"  LULUS {pesan}")
    else:
        GAGAL += 1
        print(f"  GAGAL {pesan}" + (f" ({rincian})" if rincian else ""), file=sys.stderr)
    return bool(kondisi)


# --------------------------------------------------------------------------------------------- Admin REST


class Admin:
    def __init__(self):
        self._token = None
        self._exp = 0

    def token(self):
        if time.time() > self._exp:
            r = k.request(
                "POST",
                BASE + "/realms/master/protocol/openid-connect/token",
                form={"grant_type": "password", "client_id": "admin-cli", "username": ADMIN_USER, "password": ADMIN_PASSWORD},
            )
            if r.status != 200:
                print(f"login admin master gagal: HTTP {r.status}", file=sys.stderr)
                sys.exit(1)
            self._token = r.json()["access_token"]
            self._exp = time.time() + 30
        return self._token

    def call(self, method, path, body=None):
        return k.request(method, BASE + "/admin/realms/" + path, data=body, headers={"Authorization": "Bearer " + self.token()})

    def get(self, path):
        r = self.call("GET", path)
        if r.status != 200:
            raise RuntimeError(f"GET {path} -> HTTP {r.status}")
        return r.json()


ADMIN = Admin()
USER_UJI = []  # (realm, id)


class UserUji:
    def __init__(self, realm, tag, email_terverifikasi=True):
        self.realm = realm
        self.email = f"t31-{tag}-{secrets.token_hex(4)}@uji.invalid"
        self.password = "Uji-" + secrets.token_urlsafe(12)
        self.otp_key = None
        self.recovery = []
        self._dipakai = set()
        r = ADMIN.call(
            "POST",
            f"{realm}/users",
            {
                "username": self.email,
                "email": self.email,
                "emailVerified": email_terverifikasi,
                "enabled": True,
                "credentials": [{"type": "password", "value": self.password, "temporary": False}],
            },
        )
        if r.status != 201:
            raise RuntimeError(f"gagal membuat user uji di {realm}: HTTP {r.status} {r.body[:200]}")
        self.id = r.location.rsplit("/", 1)[1]
        USER_UJI.append((realm, self.id))

    def otp(self):
        """Kode TOTP yang belum dipakai di jendela ±1 (otpPolicyCodeReusable=false, look-ahead 1)."""
        now = int(time.time() // 30)
        # Menaik saja (-1, 0, +1): Keycloak menolak kode berjendela lebih lama dari kode yang terakhir dipakai
        # sehingga pola 0, +1, lalu -1 membuat uji tidak stabil (terlihat setelah F1-T35 menggeser waktu).
        for off in (-1, 0, 1):
            if now + off > max(self._dipakai, default=-1):
                self._dipakai.add(now + off)
                return k.totp(self.otp_key, offset=off)
        time.sleep(31)
        return self.otp()

    def kode_salah(self):
        """Kode 6 digit yang pasti tidak valid di jendela ±1 saat ini (tanpa memakai slot kode valid)."""
        now = int(time.time() // 30)
        valid = {k.totp(self.otp_key, at=(now + off) * 30) for off in (-2, -1, 0, 1, 2)}
        return next(c for c in ("000000", "111111", "222222", "333333", "444444", "555555") if c not in valid)


# ------------------------------------------------------------------------------------------ alur browser


def mulai(realm, browser=None, pkce=True, klien=None):
    kl = klien or KLIEN[realm]
    b = browser or k.Browser(BASE)
    verifier, challenge = k.pkce_pair()
    q = {"client_id": kl["client_id"], "response_type": "code", "scope": "openid", "redirect_uri": kl["redirect"], "state": secrets.token_hex(8)}
    if pkce:
        q.update({"code_challenge": challenge, "code_challenge_method": "S256"})
    r = b.open(f"{BASE}/realms/{realm}/protocol/openid-connect/auth?" + urllib.parse.urlencode(q))
    return b, verifier, r


def ikuti(b, r):
    """Ikuti redirect di dalam Keycloak; berhenti di halaman atau di redirect ke klien."""
    while r.location and r.location.startswith(BASE):
        r = b.open(r.location)
    return r


def kode_dari(r, realm):
    if r.status in (302, 303) and r.location and r.location.startswith(KLIEN[realm]["redirect"]):
        q = urllib.parse.parse_qs(urllib.parse.urlsplit(r.location).query)
        return q.get("code", [None])[0]
    return None


def kirim_password(b, r, email_, password):
    p = k.Page(r.body)
    f = p.form("kc-form-login")
    if not f:
        raise RuntimeError("formulir login tidak ditemukan")
    time.sleep(JEDA)
    return ikuti(b, b.open(f["action"], {"username": email_, "password": password, "credentialId": ""}))


def kirim_otp(b, r, kode):
    p = k.Page(r.body)
    f = p.form("kc-otp-login-form")
    if not f:
        raise RuntimeError("formulir OTP tidak ditemukan")
    time.sleep(JEDA)
    return ikuti(b, b.open(f["action"], dict(f["inputs"], otp=kode)))


def daftar_mfa(b, r, user):
    """Selesaikan Configure OTP lalu kode pemulihan. Mengembalikan respons setelahnya."""
    p = k.Page(r.body)
    f = p.form("kc-totp-settings-form")
    if not f:
        raise RuntimeError("halaman Configure OTP tidak muncul")
    user.otp_key = f["inputs"]["totpSecret"]
    time.sleep(JEDA)
    r = ikuti(b, b.open(f["action"], dict(f["inputs"], totp=user.otp(), userLabel="uji")))
    return r


def simpan_kode_pemulihan(b, r, user):
    p = k.Page(r.body)
    f = p.form("kc-recovery-codes-settings-form")
    if not f:
        return None
    user.recovery = f["inputs"]["generatedRecoveryAuthnCodes"].split(",")
    return ikuti(b, b.open(f["action"], dict(f["inputs"], kcRecoveryCodesConfirmationCheck="on")))


def tukar_kode(realm, code, verifier, klien=None, secret_override=None, tanpa_secret=False):
    kl = klien or KLIEN[realm]
    form = {"grant_type": "authorization_code", "client_id": kl["client_id"], "code": code, "redirect_uri": kl["redirect"], "code_verifier": verifier}
    secret = secret_override if secret_override is not None else kl["secret"]
    if secret and not tanpa_secret:
        form["client_secret"] = secret
    return k.request("POST", f"{BASE}/realms/{realm}/protocol/openid-connect/token", form=form)


def segarkan(realm, refresh_token):
    kl = KLIEN[realm]
    form = {"grant_type": "refresh_token", "client_id": kl["client_id"], "refresh_token": refresh_token}
    if kl["secret"]:
        form["client_secret"] = kl["secret"]
    return k.request("POST", f"{BASE}/realms/{realm}/protocol/openid-connect/token", form=form)


def login_mfa_lengkap(realm, user):
    """User baru: password -> Configure OTP -> kode pemulihan -> kode otorisasi. Mengembalikan token."""
    b, verifier, r = mulai(realm)
    r = kirim_password(b, r, user.email, user.password)
    periksa(kode_dari(r, realm) is None and k.Page(r.body).form("kc-totp-settings-form") is not None,
            f"[{realm}] login pertama dengan password benar berhenti di Configure OTP, tanpa kode otorisasi")
    r = daftar_mfa(b, r, user)
    periksa(k.Page(r.body).form("kc-recovery-codes-settings-form") is not None and kode_dari(r, realm) is None,
            f"[{realm}] setelah TOTP terdaftar, kode pemulihan wajib dibuat sebelum kode otorisasi terbit (KEPUTUSAN-203)")
    u = ADMIN.get(f"{realm}/users/{user.id}")
    periksa("CONFIGURE_RECOVERY_AUTHN_CODES" in (u.get("requiredActions") or []),
            f"[{realm}] kewajiban kode pemulihan tersimpan sebagai required action user (tidak hilang bila login ditinggalkan)")
    r = simpan_kode_pemulihan(b, r, user)
    code = kode_dari(r, realm) if r is not None else None
    periksa(code is not None and len(user.recovery) == 12, f"[{realm}] kode otorisasi terbit setelah TOTP dan 12 kode pemulihan tersimpan")
    if code is None:
        return None
    t = tukar_kode(realm, code, verifier)
    periksa(t.status == 200 and "access_token" in t.json(), f"[{realm}] penukaran kode dengan PKCE berhasil", f"HTTP {t.status}")
    return t.json() if t.status == 200 else None


# ------------------------------------------------------------------------------------- pemeriksaan config


def flow_exec(realm, alias):
    return ADMIN.get(f"{realm}/authentication/flows/{urllib.parse.quote(alias)}/executions")


def cek_konfigurasi(realm):
    d = ADMIN.get(realm)
    print(f"== [{realm}] konfigurasi realm (Admin REST)")
    periksa(d.get("registrationAllowed") is False, f"[{realm}] User Registration mati (KEPUTUSAN-201, UP-73 §2.7)")
    periksa(d.get("editUsernameAllowed") is False and d.get("registrationEmailAsUsername") is True, f"[{realm}] email sebagai username, tidak dapat diubah pengguna")
    periksa(d.get("verifyEmail") is True and d.get("resetPasswordAllowed") is True, f"[{realm}] verifikasi email wajib, lupa password tersedia")
    bf = {k_: d.get(k_) for k_ in ("bruteForceProtected", "permanentLockout", "maxTemporaryLockouts", "bruteForceStrategy", "failureFactor",
                                   "waitIncrementSeconds", "maxFailureWaitSeconds", "maxDeltaTimeSeconds", "quickLoginCheckMilliSeconds",
                                   "minimumQuickLoginWaitSeconds")}
    periksa(bf == {"bruteForceProtected": True, "permanentLockout": False, "maxTemporaryLockouts": 0, "bruteForceStrategy": "LINEAR",
                   "failureFactor": 5, "waitIncrementSeconds": 900, "maxFailureWaitSeconds": 900, "maxDeltaTimeSeconds": 43200,
                   "quickLoginCheckMilliSeconds": 1000, "minimumQuickLoginWaitSeconds": 60},
            f"[{realm}] brute force sementara: 5 gagal, kunci 15 menit, linear, Max Wait 15 menit (KEPUTUSAN-190)", str(bf))
    periksa(d.get("revokeRefreshToken") is True and d.get("refreshTokenMaxReuse") == 0, f"[{realm}] Revoke Refresh Token aktif, pemakaian ulang 0")
    periksa(d.get("accessTokenLifespan") == 300, f"[{realm}] token akses 5 menit")
    periksa((d.get("ssoSessionIdleTimeout"), d.get("ssoSessionMaxLifespan")) == SESI[realm],
            f"[{realm}] sesi idle/mutlak {SESI[realm]} detik (ADR-011 butir 7)", f"{d.get('ssoSessionIdleTimeout')}/{d.get('ssoSessionMaxLifespan')}")
    periksa(d.get("actionTokenGeneratedByUserLifespan") == 3600, f"[{realm}] tautan atur ulang password 60 menit (ADR-011 butir 4)")
    otp = (d.get("otpPolicyType"), d.get("otpPolicyAlgorithm"), d.get("otpPolicyDigits"), d.get("otpPolicyPeriod"), d.get("otpPolicyCodeReusable"))
    periksa(otp == ("totp", "HmacSHA1", 6, 30, False), f"[{realm}] kebijakan OTP TOTP SHA1 6 digit 30 detik, kode tidak dapat dipakai ulang (KEPUTUSAN-85)", str(otp))
    pol = d.get("passwordPolicy") or ""
    periksa(all(x in pol for x in ("length(8)", "maxLength(128)", "notEmail(", "hashAlgorithm(argon2)")),
            f"[{realm}] kebijakan password 8-128, bukan email, hash Argon2 (KEPUTUSAN-188, sebagian)", pol)
    smtp = d.get("smtpServer") or {}
    periksa(smtp.get("starttls") == "true" and smtp.get("ssl") == "false" and smtp.get("auth") == "false"
            and "debug" not in smtp and "password" not in smtp
            and smtp.get("host") == os.environ.get("RS_KC_SMTP_HOST") and smtp.get("port") == os.environ.get("RS_KC_SMTP_PORT"),
            f"[{realm}] SMTP ke relay ADR-026: STARTTLS wajib, tanpa debug, tanpa password di realm (KEPUTUSAN-231)",
            str({x: smtp.get(x) for x in ("host", "port", "starttls", "ssl", "auth")}))
    periksa(d.get("eventsEnabled") is True and d.get("eventsExpiration") == 604800 and d.get("adminEventsEnabled") is True
            and d.get("adminEventsDetailsEnabled") is False,
            f"[{realm}] event login dan admin disimpan 7 hari, tanpa detail representasi (ADR-011 butir 10)")
    periksa(d.get("eventsListeners") == ["jboss-logging", "email"] and d.get("emailTheme") == TEMA,
            f"[{realm}] listener event email aktif dengan tema email {TEMA} (pemberitahuan password diubah, TD-27)",
            str((d.get("eventsListeners"), d.get("emailTheme"))))
    periksa((d.get("browserFlow"), d.get("resetCredentialsFlow"), d.get("directGrantFlow")) == ("rs-browser-mfa", "rs-reset-credentials-mfa", "rs-direct-grant-mfa"),
            f"[{realm}] alur browser, atur ulang, dan direct grant memakai alur MFA kustom")

    # Alur browser: OTP/kode pemulihan bila terdaftar; bila tidak, OTP REQUIRED (Configure OTP dipaksa).
    bx = flow_exec(realm, "rs-browser-mfa")
    prov = [(e.get("providerId"), e.get("requirement"), e.get("displayName")) for e in bx]
    periksa(("auth-otp-form", "REQUIRED", "OTP Form") in prov and ("auth-recovery-authn-code-form", "ALTERNATIVE", "Recovery Authentication Code Form") in prov
            and not any(p_ == "auth-otp-form" and r_ == "DISABLED" for p_, r_, _ in prov)
            and not any(p_ in ("identity-provider-redirector", "auth-spnego") for p_, _, _ in prov),
            f"[{realm}] alur browser memuat OTP REQUIRED (cabang belum terdaftar) dan kode pemulihan, tanpa IdP/Kerberos", str(prov))
    rx = flow_exec(realm, "rs-reset-credentials-mfa")
    rprov = [e.get("providerId") for e in rx]
    periksa("reset-otp" not in rprov and "auth-otp-form" in rprov and "reset-password" in rprov
            and rprov.index("auth-otp-form") < rprov.index("reset-password"),
            f"[{realm}] alur atur ulang: OTP diminta sebelum password baru, tanpa langkah Reset OTP (KEPUTUSAN-199)", str(rprov))
    em = next((e for e in rx if e.get("providerId") == "reset-credential-email"), {})
    cfg = ADMIN.get(f"{realm}/authentication/config/{em['authenticationConfig']}") if em.get("authenticationConfig") else {}
    periksa((cfg.get("config") or {}).get("force-login") == "true",
            f"[{realm}] atur ulang password: 'Force login after reset' aktif, pengguna tidak otomatis masuk (US-103 AC4d)", str(cfg.get("config")))
    dx = flow_exec(realm, "rs-direct-grant-mfa")
    periksa(any(e.get("providerId") == "direct-grant-validate-otp" and e.get("requirement") == "REQUIRED" for e in dx),
            f"[{realm}] alur direct grant mewajibkan OTP tanpa syarat")

    ra = {a["alias"]: a for a in ADMIN.get(f"{realm}/authentication/required-actions")}
    totp = ra.get("CONFIGURE_TOTP", {})
    periksa(totp.get("enabled") is True and (totp.get("config") or {}).get("add-recovery-codes") == "true",
            f"[{realm}] Configure OTP aktif dengan 'Add Recovery Codes' (KEPUTUSAN-203)")
    periksa(ra.get("CONFIGURE_RECOVERY_AUTHN_CODES", {}).get("enabled") is True, f"[{realm}] required action kode pemulihan aktif")
    periksa(ra.get("UPDATE_EMAIL", {}).get("enabled") is False, f"[{realm}] Update Email lewat Keycloak mati (ADR-011 butir 6)")
    periksa(ra.get("VERIFY_EMAIL", {}).get("enabled") is True
            and ra.get("VERIFY_EMAIL", {}).get("priority", 999) < totp.get("priority", -1),
            f"[{realm}] Verify Email berjalan sebelum Configure OTP (prioritas lebih kecil; cegah pre-account hijack)")

    up = ADMIN.get(f"{realm}/users/profile")
    attrs = {a["name"]: a for a in up.get("attributes", [])}
    periksa(attrs.get("email", {}).get("permissions", {}).get("edit") == ["admin"]
            and attrs.get("username", {}).get("permissions", {}).get("edit") == ["admin"]
            and "firstName" not in attrs and "lastName" not in attrs,
            f"[{realm}] profil user: email/username hanya diubah admin (lewat backend), Keycloak hanya menyimpan email (ADR-011 butir 6)")

    klien = ADMIN.get(f"{realm}/clients")
    periksa(all(c.get("directAccessGrantsEnabled") is not True for c in klien) and all(c.get("implicitFlowEnabled") is not True for c in klien),
            f"[{realm}] tidak ada klien dengan grant password atau implicit",
            str([c["clientId"] for c in klien if c.get("directAccessGrantsEnabled") or c.get("implicitFlowEnabled")]))
    target = [(KLIEN[realm]["client_id"], KLIEN[realm]["redirect"], KLIEN[realm]["secret"] is None)] + KLIEN_LAIN[realm]
    for cid, redirect, publik in target:
        c = next((c for c in klien if c["clientId"] == cid), None)
        ok = (c is not None and c.get("publicClient") is publik and c.get("standardFlowEnabled") is True
              and (c.get("attributes") or {}).get("pkce.code.challenge.method") == "S256"
              and c.get("redirectUris") == [redirect] and c.get("serviceAccountsEnabled") is False
              and c.get("fullScopeAllowed") is False)
        periksa(ok, f"[{realm}] klien {cid}: {'publik' if publik else 'rahasia'}, Authorization Code + PKCE S256, redirect tepat satu")
        if c is not None and not publik:
            s = ADMIN.get(f"{realm}/clients/{c['id']}/client-secret")
            want = os.environ.get("RS_KC_" + cid.replace("rs-", "").replace("-", "_").upper() + "_SECRET")
            periksa(s.get("value") == want, f"[{realm}] klien {cid}: secret berasal dari variabel lingkungan saat penerapan")


# ------------------------------------------------------------------------------------- pemeriksaan perilaku


def cek_pendaftaran_mati(realm):
    kl = KLIEN[realm]
    b, _, r = mulai(realm)
    p = k.Page(r.body)
    periksa(p.form("kc-form-login") is not None and not any("registration" in (h or "") for _, h in p.links),
            f"[{realm}] halaman login tanpa tautan pendaftaran")
    v, c = k.pkce_pair()
    q = urllib.parse.urlencode({"client_id": kl["client_id"], "response_type": "code", "scope": "openid", "redirect_uri": kl["redirect"],
                                "code_challenge": c, "code_challenge_method": "S256"})
    r = ikuti(b, b.open(f"{BASE}/realms/{realm}/protocol/openid-connect/registrations?" + q))
    p = k.Page(r.body)
    periksa(p.form("kc-register-form") is None, f"[{realm}] endpoint pendaftaran bawaan tidak menampilkan formulir pendaftaran", f"HTTP {r.status}")


def cek_pkce_wajib(realm):
    for cid, redirect in [(KLIEN[realm]["client_id"], KLIEN[realm]["redirect"])] + [(c, r) for c, r, _ in KLIEN_LAIN[realm]]:
        _, _, r = mulai(realm, pkce=False, klien={"client_id": cid, "redirect": redirect})
        loc = r.location or ""
        periksa(r.status in (302, 303) and "error=invalid_request" in loc and "code=" not in loc,
                f"[{realm}] klien {cid}: permintaan otorisasi tanpa PKCE ditolak", f"HTTP {r.status}")


def cek_mfa_dan_token(realm):
    print(f"== [{realm}] MFA wajib, kode pemulihan, token")
    user = UserUji(realm, "mfa")
    tok = login_mfa_lengkap(realm, user)
    if tok is None:
        return None
    cek_klaim_token(realm, tok)
    # Rotasi token penyegar: token lama ditolak setelah dipakai sekali.
    r1 = segarkan(realm, tok["refresh_token"])
    r2 = segarkan(realm, tok["refresh_token"])
    periksa(r1.status == 200 and r2.status == 400 and (r2.json() or {}).get("error") == "invalid_grant",
            f"[{realm}] token penyegar sekali pakai (Revoke Refresh Token)", f"{r1.status}/{r2.status}")
    # Login berikutnya: password saja tidak cukup.
    b, verifier, r = mulai(realm)
    r = kirim_password(b, r, user.email, user.password)
    periksa(kode_dari(r, realm) is None and k.Page(r.body).form("kc-otp-login-form") is not None,
            f"[{realm}] login dengan TOTP terdaftar: password benar berhenti di formulir OTP")
    r = kirim_otp(b, r, user.kode_salah())
    periksa(kode_dari(r, realm) is None and k.Page(r.body).form("kc-otp-login-form") is not None,
            f"[{realm}] OTP salah tidak menerbitkan kode otorisasi")
    r = kirim_otp(b, r, user.otp())
    code = kode_dari(r, realm)
    periksa(code is not None, f"[{realm}] OTP benar menerbitkan kode otorisasi")
    if code:
        if KLIEN[realm]["secret"]:
            t = tukar_kode(realm, code, verifier, tanpa_secret=True)
            periksa(t.status in (400, 401) and "access_token" not in (t.json() or {}), f"[{realm}] klien rahasia: penukaran kode tanpa secret ditolak", f"HTTP {t.status}")
        else:
            t = tukar_kode(realm, code, "x" * 50)
            periksa(t.status == 400 and (t.json() or {}).get("error") == "invalid_grant", f"[{realm}] code_verifier salah ditolak (PKCE)", f"HTTP {t.status}")
    return user


def buka_kode_pemulihan(realm, user):
    """Login password lalu 'pakai cara lain' sampai formulir kode pemulihan. Mengembalikan (browser, respons)
    atau (browser, None) bila jalur itu tidak ditawarkan."""
    flows = flow_exec(realm, "rs-browser-mfa")
    rc_exec = next(e["id"] for e in flows if e.get("providerId") == "auth-recovery-authn-code-form")
    b, verifier, r = mulai(realm)
    r = kirim_password(b, r, user.email, user.password)
    f = k.Page(r.body).form("kc-select-try-another-way-form")
    if not f:
        return b, None
    r = ikuti(b, b.open(f["action"], f["inputs"]))
    sel = [f_ for f_ in k.Page(r.body).forms if f_["id"] == "kc-select-credential-form" and f_["inputs"].get("authenticationExecution") == rc_exec]
    if not sel:
        return b, None
    r = ikuti(b, b.open(sel[0]["action"], sel[0]["inputs"]))
    return b, r


def cek_kode_pemulihan(realm, user):
    print(f"== [{realm}] masuk dengan kode pemulihan")
    pilih_kode_pemulihan = lambda: buka_kode_pemulihan(realm, user)  # noqa: E731

    b, r = pilih_kode_pemulihan()
    f = k.Page(r.body).form("kc-recovery-code-login-form") if r is not None else None
    periksa(f is not None, f"[{realm}] 'coba cara lain' menawarkan kode pemulihan")
    if not f:
        return
    pertama = user.recovery[0]
    time.sleep(JEDA)
    r = ikuti(b, b.open(f["action"], dict(f["inputs"], recoveryCodeInput=pertama)))
    periksa(kode_dari(r, realm) is not None, f"[{realm}] kode pemulihan pertama menerbitkan kode otorisasi")
    b, r = pilih_kode_pemulihan()
    f = k.Page(r.body).form("kc-recovery-code-login-form") if r is not None else None
    if f:
        time.sleep(JEDA)
        r = ikuti(b, b.open(f["action"], dict(f["inputs"], recoveryCodeInput=pertama)))
    periksa(f is not None and kode_dari(r, realm) is None, f"[{realm}] kode pemulihan yang sudah dipakai ditolak")


# ------------------------------------------------------------------------- MFA pasien (F1-T72, F1-T55)


def user_mfa(realm, tag):
    """User baru yang sudah menyelesaikan pendaftaran TOTP dan menyimpan kode pemulihan."""
    user = UserUji(realm, tag)
    return user if login_mfa_lengkap(realm, user) is not None else None


def percobaan_gagal(realm, user_id):
    return ADMIN.get(f"{realm}/attack-detection/brute-force/users/{user_id}")


def galat_event(realm, user_id):
    """Daftar `error` event LOGIN_ERROR milik user (Admin REST)."""
    ev = ADMIN.get(f"{realm}/events?type=LOGIN_ERROR&user={user_id}&max=100") or []
    return [e.get("error") for e in ev]


def cek_otp_dan_kode_pemulihan_salah_dikunci(realm):
    """US-103 AC3 untuk faktor kedua (F1-T72, `[SRC-273]`) dan E-02 (F1-T55): OTP salah dan kode pemulihan salah
    dihitung penguncian; saat terkunci OTP benar pun tidak menerbitkan kode otorisasi."""
    print(f"== [{realm}] OTP dan kode pemulihan salah dihitung penguncian, E-02")
    u = user_mfa(realm, "otpkunci")
    if u is None:
        return
    for _ in range(5):
        b, _, r = mulai(realm)
        r = kirim_password(b, r, u.email, u.password)
        r = kirim_otp(b, r, u.kode_salah())
    bf = percobaan_gagal(realm, u.id)
    periksa(bf.get("disabled") is True and bf.get("numFailures") == 5, f"[{realm}] 5 OTP salah (password benar) mengunci akun", str(bf))
    b, verifier, r = mulai(realm)
    r = kirim_password(b, r, u.email, u.password)
    teks = pesan_galat(r)
    periksa(kode_dari(r, realm) is None and k.Page(r.body).form("kc-otp-login-form") is None and teks == PESAN_SERAGAM,
            f"[{realm}] akun terkunci karena OTP: password benar tidak menuju formulir OTP, pesan seragam", repr(teks))
    errs = galat_event(realm, u.id)
    periksa("user_temporarily_disabled" in errs and errs.count("invalid_user_credentials") >= 5,
            f"[{realm}] E-02 tercatat: LOGIN_ERROR invalid_user_credentials dari OTP salah dan user_temporarily_disabled saat terkunci", str(errs[:8]))

    v = user_mfa(realm, "rckunci")
    if v is None:
        return
    for _ in range(5):
        b, r = buka_kode_pemulihan(realm, v)
        f = k.Page(r.body).form("kc-recovery-code-login-form") if r is not None else None
        if not f:
            periksa(False, f"[{realm}] formulir kode pemulihan tersedia untuk uji penguncian")
            return
        time.sleep(JEDA)
        ikuti(b, b.open(f["action"], dict(f["inputs"], recoveryCodeInput="ZZZZZZZZZZZZ")))
    bf = percobaan_gagal(realm, v.id)
    periksa(bf.get("disabled") is True and bf.get("numFailures") == 5, f"[{realm}] 5 kode pemulihan salah mengunci akun", str(bf))
    b, r = buka_kode_pemulihan(realm, v)
    periksa(r is None or k.Page(r.body).form("kc-recovery-code-login-form") is None,
            f"[{realm}] akun terkunci karena kode pemulihan: formulir kode pemulihan tidak tersedia walau password benar")


def cek_batal_pendaftaran_totp(realm):
    """US-101 AC5, US-103 AC1b (F1-T72): akun yang meninggalkan pendaftaran TOTP atau kode pemulihan tidak masuk."""
    print(f"== [{realm}] membatalkan pendaftaran TOTP atau kode pemulihan tidak memberi akses")
    u = UserUji(realm, "batal")
    b, _, r = mulai(realm)
    r = kirim_password(b, r, u.email, u.password)
    periksa(k.Page(r.body).form("kc-totp-settings-form") is not None, f"[{realm}] login pertama berhenti di pendaftaran TOTP")
    f = k.Page(r.body).form("kc-totp-settings-form")
    time.sleep(JEDA)
    r2 = ikuti(b, b.open(f["action"], dict(f["inputs"], totp="000000", userLabel="uji")))
    periksa(kode_dari(r2, realm) is None and k.Page(r2.body).form("kc-totp-settings-form") is not None,
            f"[{realm}] kode TOTP salah saat pendaftaran tidak menyelesaikan login")
    _, _, r3 = mulai(realm, browser=b)
    periksa(kode_dari(r3, realm) is None, f"[{realm}] permintaan otorisasi baru di peramban yang sama tidak langsung menerbitkan kode")
    b2, _, r4 = mulai(realm)
    r4 = kirim_password(b2, r4, u.email, u.password)
    periksa(kode_dari(r4, realm) is None and k.Page(r4.body).form("kc-totp-settings-form") is not None,
            f"[{realm}] login ulang setelah meninggalkan pendaftaran TOTP kembali ke pendaftaran TOTP, tanpa kode otorisasi")
    ada_otp = [c for c in ADMIN.get(f"{realm}/users/{u.id}/credentials") if c["type"] == "otp"]
    periksa(not ada_otp, f"[{realm}] tidak ada kredensial OTP tersimpan setelah pendaftaran ditinggalkan")

    # TOTP terdaftar, kode pemulihan ditinggalkan: login berikutnya OTP lalu kode pemulihan wajib lagi.
    r5 = daftar_mfa(b2, r4, u)
    periksa(k.Page(r5.body).form("kc-recovery-codes-settings-form") is not None, f"[{realm}] setelah TOTP, halaman kode pemulihan muncul")
    b3, _, r6 = mulai(realm)
    r6 = kirim_password(b3, r6, u.email, u.password)
    r6 = kirim_otp(b3, r6, u.otp())
    periksa(kode_dari(r6, realm) is None and k.Page(r6.body).form("kc-recovery-codes-settings-form") is not None,
            f"[{realm}] kode pemulihan ditinggalkan: setelah OTP benar, kode pemulihan masih wajib dan tidak ada kode otorisasi (KEPUTUSAN-203)")


def cek_klaim_token(realm, token):
    """F1-T72: klaim tingkat autentikasi pada token akses (hasil dicetak untuk reviewer keamanan)."""
    import base64
    bagian = token["access_token"].split(".")[1]
    klaim = json.loads(base64.urlsafe_b64decode(bagian + "=" * (-len(bagian) % 4)))
    print(f"  INFO [{realm}] klaim token akses setelah password+OTP: acr={klaim.get('acr')!r}, amr={klaim.get('amr')!r}, "
          f"auth_time={'ada' if 'auth_time' in klaim else 'tidak ada'}; kunci klaim={sorted(klaim)}")


def cek_argon2_dan_kebijakan(realm):
    print(f"== [{realm}] Argon2 dan kebijakan password")
    user = UserUji(realm, "hash")
    cred = next(c for c in ADMIN.get(f"{realm}/users/{user.id}/credentials") if c["type"] == "password")
    data = json.loads(cred.get("credentialData") or "{}")
    ap = data.get("additionalParameters") or {}
    periksa(data.get("algorithm") == "argon2" and data.get("hashIterations") == 2 and ap.get("memory") == ["19456"] and ap.get("parallelism") == ["1"]
            and ap.get("type") == ["id"],
            f"[{realm}] hash password Argon2id 19456 KB / 2 iterasi / paralelisme 1 (ADR-011 butir 4)",
            str({"algorithm": data.get("algorithm"), "iterations": data.get("hashIterations"), **{x: ap.get(x) for x in ("memory", "parallelism", "type")}}))
    for nilai, alasan in (("Ab-1234", "7 karakter"), ("A" * 129, "129 karakter"), (user.email, "sama dengan email")):
        r = ADMIN.call("PUT", f"{realm}/users/{user.id}/reset-password", {"type": "password", "value": nilai, "temporary": False})
        periksa(r.status == 400, f"[{realm}] password {alasan} ditolak kebijakan", f"HTTP {r.status}")
    r = ADMIN.call("PUT", f"{realm}/users/{user.id}/reset-password", {"type": "password", "value": "x" * 128, "temporary": False})
    periksa(r.status == 204, f"[{realm}] password 128 karakter diterima", f"HTTP {r.status}")


def pesan_galat(r):
    """Teks galat halaman login. Tema bawaan menaruhnya di elemen berkelas `kc-feedback-text`
    (galat isian untuk password salah, banner untuk akun nonaktif)."""
    found = re.findall(r'class="[^"]*\bkc-feedback-text\b[^"]*"[^>]*>([^<]*)<', r.body)
    teks = [re.sub(r"\s+", " ", k.unescape(t)).strip() for t in found]
    teks = [t for t in teks if t]
    return teks[0] if len(teks) == 1 else (teks or None)


def cek_penguncian_dan_pesan(realm):
    print(f"== [{realm}] penguncian 5 gagal dan pesan seragam")
    korban = UserUji(realm, "kunci")
    pesan = {}
    status = {}
    halaman = {}
    for i in range(5):
        b, _, r = mulai(realm)
        r = kirim_password(b, r, korban.email, "Salah-" + secrets.token_hex(6))
        if i == 0:
            pesan["password salah"] = pesan_galat(r)
            status["password salah"] = r.status
            halaman["password salah"] = r
    bf = ADMIN.get(f"{realm}/attack-detection/brute-force/users/{korban.id}")
    periksa(bf.get("disabled") is True and bf.get("numFailures") == 5, f"[{realm}] setelah 5 gagal akun terkunci sementara (attack detection)", str(bf))
    b, _, r = mulai(realm)
    r = kirim_password(b, r, korban.email, korban.password)
    pesan["terkunci, password benar"] = pesan_galat(r)
    status["terkunci, password benar"] = r.status
    halaman["terkunci, password benar"] = r
    periksa(kode_dari(r, realm) is None and k.Page(r.body).form("kc-otp-login-form") is None and k.Page(r.body).form("kc-totp-settings-form") is None,
            f"[{realm}] akun terkunci: password benar tidak melanjutkan login")
    bf2 = ADMIN.get(f"{realm}/attack-detection/brute-force/users/{korban.id}")
    periksa(bf2.get("numFailures") == 5, f"[{realm}] percobaan selama terkunci tidak menambah hitungan", str(bf2.get("numFailures")))

    b, _, r = mulai(realm)
    r = kirim_password(b, r, f"t31-tidak-ada-{secrets.token_hex(4)}@uji.invalid", "Apa-saja-12345")
    pesan["akun tidak ada"] = pesan_galat(r)
    status["akun tidak ada"] = r.status
    halaman["akun tidak ada"] = r

    nonaktif = UserUji(realm, "nonaktif")
    ADMIN.call("PUT", f"{realm}/users/{nonaktif.id}", {"enabled": False})
    b, _, r = mulai(realm)
    r = kirim_password(b, r, nonaktif.email, nonaktif.password)
    pesan["akun nonaktif, password benar"] = pesan_galat(r)
    status["akun nonaktif, password benar"] = r.status
    halaman["akun nonaktif, password benar"] = r

    periksa(all(v == PESAN_SERAGAM for v in pesan.values()), f"[{realm}] pesan login gagal identik kata per kata untuk 4 kondisi (US-103 AC2)",
            "; ".join(f"{a}={b_!r}" for a, b_ in pesan.items()))
    periksa(len(set(status.values())) == 1, f"[{realm}] kode HTTP halaman login gagal sama untuk 4 kondisi", str(status))
    cek_dom_seragam(realm, halaman)


# ------------------------------------------------------------------------------------- tema login (F1-T35)

TEMA = "ruang-sehat"
RAIZ = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
HOST_BASE = urllib.parse.urlsplit(BASE).netloc
CSP_BAWAAN = "frame-src 'self'; frame-ancestors 'self'; object-src 'none';"


def normalisasi(body):
    """Buang bagian yang memang berbeda per sesi/percobaan (nilai isian, URL bersesi, isi skrip, spasi);
    sisanya adalah struktur DOM dan teks yang harus identik untuk keempat kegagalan login."""
    b = re.sub(r"<script\b.*?</script>", "<script/>", body, flags=re.S | re.I)
    b = re.sub(r'\b(action|href)="[^"]*(?:session_code|tab_id|execution|client_data)[^"]*"', r'\1="#"', b)
    b = re.sub(r'(<input\b[^>]*?)\bvalue="[^"]*"', r'\1value=""', b)
    b = re.sub(r"\s+", " ", b)
    return re.sub(r">\s*<", ">\n<", b).strip()


def cek_dom_seragam(realm, halaman):
    """UP-73 opsi B, TD-16: DOM halaman login gagal identik untuk password salah, akun tidak ada, terkunci,
    dan nonaktif. Bukan hanya teks: banner vs galat isian pada tema bawaan membedakan status akun."""
    norm = {n: normalisasi(r.body) for n, r in halaman.items()}
    acuan_nama = "password salah"
    acuan = norm[acuan_nama]
    for nama, dom in norm.items():
        if nama == acuan_nama:
            continue
        sama = dom == acuan
        rincian = ""
        if not sama:
            rincian = "\n".join(list(difflib.unified_diff(acuan.splitlines(), dom.splitlines(), acuan_nama, nama, lineterm="", n=1))[:30])
        periksa(sama, f"[{realm}] DOM halaman login gagal '{nama}' identik dengan '{acuan_nama}' setelah normalisasi (TD-16)", rincian)
    for nama, r in halaman.items():
        b = r.body
        periksa(b.count('id="kc-login-error"') == 1 and 'role="alert"' in b and 'aria-invalid="true"' not in b
                and b.count("kc-feedback-text") == 1 and 'id="input-error-username"' not in b,
                f"[{realm}] '{nama}': satu blok galat role=alert, tanpa galat isian per-kolom")
        periksa('lang="id"' in b, f"[{realm}] '{nama}': halaman berbahasa Indonesia (lang=id)")


def url_luar(teks):
    """URL absolut ke host selain Keycloak sendiri. Namespace XML SVG bukan permintaan jaringan."""
    ketemu = []
    # Komentar JS baris penuh (mis. rujukan bugzilla di template.ftl bawaan) bukan permintaan jaringan.
    teks = re.sub(r"^[ \t]*//[^\n]*$", "", teks, flags=re.M)
    for m in re.finditer(r"(?:https?:)?//([A-Za-z0-9.-]+\.[A-Za-z]{2,}|\d+\.\d+\.\d+\.\d+)(?::\d+)?([^\s\"'<>)]*)", teks):
        host = m.group(1)
        if host == "www.w3.org" and re.match(r"/(2000/svg|1999/xlink|1999/xhtml)", m.group(2)):
            continue
        penuh = m.group(0) if m.group(0).startswith("http") else "http:" + m.group(0)
        if urllib.parse.urlsplit(penuh).netloc == HOST_BASE:
            continue
        ketemu.append(m.group(0) + "  (konteks: " + teks[max(0, m.start() - 12):m.start()].strip() + ")")
    return ketemu


def cek_tema(realm):
    print(f"== [{realm}] tema login {TEMA} (F1-T35)")
    d = ADMIN.get(realm)
    periksa(d.get("loginTheme") == TEMA, f"[{realm}] realm memakai tema login {TEMA}", str(d.get("loginTheme")))
    periksa(d.get("internationalizationEnabled") is True and d.get("supportedLocales") == ["id"] and d.get("defaultLocale") == "id",
            f"[{realm}] satu bahasa: Indonesia (tanpa pemilih bahasa)")
    b, _, r = mulai(realm)
    halaman = r.body
    periksa(r.status == 200 and f"/login/{TEMA}/" in halaman and "css/ruang-sehat.css" in halaman,
            f"[{realm}] halaman login memuat sumber daya tema {TEMA} (bukan tema bawaan)", f"HTTP {r.status}")
    periksa('lang="id"' in halaman and "<title>Masuk ke Ruang Sehat" in halaman and "login-select-toggle" not in halaman,
            f"[{realm}] halaman login berbahasa Indonesia, tanpa pemilih bahasa")
    periksa("Ruang Sehat" in (k.Page(halaman).text("kc-header-wrapper") or ""), f"[{realm}] kepala halaman memuat nama produk")
    periksa("pf-v5-theme-dark" not in halaman and "prefers-color-scheme" not in halaman, f"[{realm}] mode gelap tidak aktif (rilis 1)")
    # CSP Keycloak tetap ketat: tema dan realm tidak melonggarkannya.
    csp = r.headers.get("Content-Security-Policy") or ""
    periksa(csp == CSP_BAWAAN, f"[{realm}] CSP halaman login tetap bawaan Keycloak (tanpa pelonggaran)", csp)
    luar = url_luar(halaman)
    periksa(not luar, f"[{realm}] HTML halaman login tanpa URL ke domain luar", "; ".join(luar[:5]))
    # Tidak ada permintaan ke domain luar: setiap stylesheet/skrip/modul yang ditautkan dari halaman.
    aset = set()
    for m in re.finditer(r'<(?:link|script)\b[^>]*?\b(?:href|src)="([^"]+)"', halaman):
        aset.add(k.unescape(m.group(1)))
    for m in re.finditer(r'"rfc4648":\s*"([^"]+)"', halaman):
        aset.add(k.unescape(m.group(1)))
    periksa(len(aset) >= 3, f"[{realm}] aset halaman login terdaftar", str(len(aset)))
    for a in sorted(aset):
        a_abs = urllib.parse.urljoin(BASE + "/", a)
        nama = urllib.parse.urlsplit(a).path.rsplit("/", 1)[-1]
        if urllib.parse.urlsplit(a_abs).netloc != HOST_BASE:
            periksa(False, f"[{realm}] aset {a} berasal dari domain luar")
            continue
        ra = k.request("GET", a_abs)
        periksa(ra.status == 200, f"[{realm}] aset {nama} dapat diambil dari Keycloak", f"HTTP {ra.status}")
        l2 = url_luar(ra.body)
        periksa(not l2, f"[{realm}] aset {nama} tanpa URL ke domain luar", "; ".join(l2[:5]))
        if a_abs.endswith(".css"):
            for u in re.findall(r"url\(\s*['\"]?([^'\")]+)", ra.body):
                if not u.startswith("data:"):
                    uabs = urllib.parse.urljoin(a_abs, u)
                    periksa(urllib.parse.urlsplit(uabs).netloc == HOST_BASE, f"[{realm}] url() di {nama} menuju Keycloak sendiri", uabs)
    # Token desain: tokens.css yang disajikan = web/packages/ui/src/tokens.css.
    css_tema = [a for a in aset if a.endswith("/css/tokens.css")]
    if periksa(len(css_tema) == 1, f"[{realm}] tokens.css dimuat halaman login"):
        sajian = k.request("GET", urllib.parse.urljoin(BASE + "/", css_tema[0])).body
        with open(os.path.join(RAIZ, "web", "packages", "ui", "src", "tokens.css"), encoding="utf-8") as fh:
            sumber = fh.read()
        periksa(sajian == sumber, f"[{realm}] tokens.css tema identik dengan web/packages/ui/src/tokens.css (token desain)")
    # Halaman Lupa password.
    m = re.search(r'href="([^"]*reset-credentials[^"]*)"', halaman)
    if periksa(m is not None, f"[{realm}] tautan Lupa password ada"):
        rr = b.open(k.unescape(m.group(1)))
        periksa(rr.status == 200 and "Lupa password" in rr.body and 'lang="id"' in rr.body and "Kirim" in rr.body,
                f"[{realm}] halaman Lupa password berbahasa Indonesia", f"HTTP {rr.status}")
        periksa(not url_luar(rr.body), f"[{realm}] halaman Lupa password tanpa URL ke domain luar", "; ".join(url_luar(rr.body)[:5]))


def cek_halaman_mfa_tema(realm):
    """Tema di halaman OTP, kode pemulihan, dan pendaftaran TOTP: berbahasa Indonesia dan tanpa domain luar.
    Memakai user baru supaya tidak mengganggu skenario lain."""
    print(f"== [{realm}] tema: pendaftaran TOTP, kode pemulihan, OTP")
    user = UserUji(realm, "tema")
    b, _, r = mulai(realm)
    r = kirim_password(b, r, user.email, user.password)
    periksa(k.Page(r.body).form("kc-totp-settings-form") is not None and "Siapkan aplikasi autentikator" in r.body and 'lang="id"' in r.body,
            f"[{realm}] halaman pendaftaran TOTP bertema dan berbahasa Indonesia")
    periksa(not url_luar(r.body), f"[{realm}] halaman pendaftaran TOTP tanpa URL ke domain luar", "; ".join(url_luar(r.body)[:5]))
    # F1-T73: kode QR (gambar data: dari Keycloak sendiri) dan kunci teks pada mode manual.
    periksa(re.search(r'<img id="kc-totp-secret-qr-code" src="data:image/png;base64,[^"]{200,}"', r.body) is not None
            and "kc-totp-secret-key" not in r.body and "mode-manual" in r.body and "Tidak bisa memindai?" in r.body,
            f"[{realm}] pendaftaran TOTP menampilkan kode QR dan tautan 'Tidak bisa memindai?' berbahasa Indonesia")
    tautan = next((h for i, h in k.Page(r.body).links if i == "mode-manual"), None)
    if tautan:
        rm = ikuti(b, b.open(urllib.parse.urljoin(BASE, tautan)))
        kunci = k.Page(rm.body).text("kc-totp-secret-key") or ""
        periksa(re.fullmatch(r"[A-Z2-7]{4}( [A-Z2-7]{4}){3,}", kunci) is not None and "Algoritma" in rm.body and "Pindai kode QR" in rm.body
                and k.Page(rm.body).form("kc-totp-settings-form") is not None,
                f"[{realm}] mode manual menampilkan kunci teks (basis 32, berkelompok) dan tautan kembali ke kode QR", kunci)
        periksa(user.otp_key is None and k.Page(rm.body).form("kc-totp-settings-form")["inputs"].get("totpSecret", "").replace(" ", "")
                == k.Page(r.body).form("kc-totp-settings-form")["inputs"].get("totpSecret", "").replace(" ", ""),
                f"[{realm}] kunci teks dan kode QR memakai rahasia yang sama (formulir tersembunyi identik)")
    r = daftar_mfa(b, r, user)
    periksa(k.Page(r.body).form("kc-recovery-codes-settings-form") is not None and "Kode pemulihan hanya ditampilkan sekali" in r.body,
            f"[{realm}] halaman kode pemulihan bertema dan berbahasa Indonesia")
    periksa(not url_luar(r.body), f"[{realm}] halaman kode pemulihan tanpa URL ke domain luar", "; ".join(url_luar(r.body)[:5]))
    kode_tampil = re.findall(r"<li>([A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4})</li>", r.body)
    kode_form = k.Page(r.body).form("kc-recovery-codes-settings-form")["inputs"].get("generatedRecoveryAuthnCodes", "").split(",")
    periksa(len(kode_tampil) == 12 and [c.replace("-", "") for c in kode_tampil] == kode_form and len(set(kode_tampil)) == 12,
            f"[{realm}] halaman kode pemulihan menampilkan 12 kode berbeda yang sama dengan yang disimpan", str(len(kode_tampil)))
    periksa("Warning alert" not in r.body and "en-US" not in r.body and "Peringatan" in r.body and "kcRecoveryCodesConfirmationCheck" in r.body,
            f"[{realm}] halaman kode pemulihan: teks pembaca layar dan tanggal unduhan berbahasa Indonesia, ada kotak konfirmasi simpan")
    simpan_kode_pemulihan(b, r, user)
    b, _, r = mulai(realm)
    r = kirim_password(b, r, user.email, user.password)
    periksa("kc-cs-hint" in r.body and "Layanan Pelanggan" in r.body and "Kehilangan perangkat autentikator dan kode pemulihan" in r.body,
            f"[{realm}] layar kode OTP memuat petunjuk menghubungi Layanan Pelanggan (US-103 AC8)")
    periksa(k.Page(r.body).form("kc-otp-login-form") is not None and "Kode OTP" in r.body and "Pakai cara lain" in r.body and 'lang="id"' in r.body,
            f"[{realm}] halaman OTP berbahasa Indonesia dengan tautan kode pemulihan")
    br, rr = buka_kode_pemulihan(realm, user)
    periksa(rr is not None and "kc-cs-hint" in rr.body and "Layanan Pelanggan" in rr.body and "Kode pemulihan #" in rr.body,
            f"[{realm}] layar kode pemulihan memuat petunjuk Layanan Pelanggan dan label berbahasa Indonesia")
    periksa(not url_luar(r.body), f"[{realm}] halaman OTP tanpa URL ke domain luar", "; ".join(url_luar(r.body)[:5]))
    r = kirim_otp(b, r, user.kode_salah())
    periksa(k.Page(r.body).form("kc-otp-login-form") is not None and "Kode tidak sesuai." in r.body,
            f"[{realm}] kode OTP salah: pesan berbahasa Indonesia")


def baca_email(setelah):
    """Pesan baru di penangkap SMTP sejak daftar `setelah`."""
    for _ in range(20):
        ls = subprocess.run(["docker", "exec", SINK, "sh", "-c", "ls /tmp/mail 2>/dev/null"], capture_output=True, text=True).stdout.split()
        baru = [f for f in ls if f not in setelah]
        if baru:
            raw = subprocess.run(["docker", "exec", SINK, "cat", "/tmp/mail/" + baru[0]], capture_output=True, text=True).stdout
            return baru[0], raw
        time.sleep(0.5)
    return None, None


def daftar_email():
    return subprocess.run(["docker", "exec", SINK, "sh", "-c", "ls /tmp/mail 2>/dev/null"], capture_output=True, text=True).stdout.split()


def tautan_atur_ulang(raw):
    m = email.message_from_string(raw)
    for part in m.walk():
        if part.get_content_type() == "text/plain":
            txt = part.get_payload(decode=True).decode("utf-8", "replace")
            found = re.search(r"(https?://\S+/login-actions/action-token\?\S+)", txt)
            if found:
                return found.group(1)
    return None


def klaim(token_akses):
    import base64
    bagian = token_akses.split(".")[1]
    return json.loads(base64.urlsafe_b64decode(bagian + "=" * (-len(bagian) % 4)))


def sesi_aktif(realm, user):
    """Sesi login lengkap (password + OTP) beserta token; None bila gagal."""
    b, verifier, r = mulai(realm)
    r = kirim_password(b, r, user.email, user.password)
    r = kirim_otp(b, r, user.otp())
    code = kode_dari(r, realm)
    if code is None:
        return None
    t = tukar_kode(realm, code, verifier)
    return t.json() if t.status == 200 else None


def teks_email(raw):
    """(subjek terurai, gabungan bagian teks dan HTML terurai) dari pesan MIME."""
    m = email.message_from_string(raw)
    subjek = str(email.header.make_header(email.header.decode_header(m.get("Subject", ""))))
    isi = []
    for part in m.walk():
        if part.get_content_maintype() == "text":
            isi.append(part.get_payload(decode=True).decode(part.get_content_charset() or "utf-8", "replace"))
    return subjek, "\n".join(isi)


SUBJEK_PASSWORD_DIUBAH = "Password akun Ruang Sehat Anda telah diubah"


def cek_atur_ulang_tidak_melewati_otp(realm, user):
    print(f"== [{realm}] atur ulang password tidak melewati OTP (US-103 AC4e) dan mengakhiri semua sesi (AC4d, TD-27)")
    # Konfigurasi relay produksi tetap diperiksa di cek_konfigurasi.
    asli = smtp_ke_penangkap(realm)
    try:
        # TD-27: sesi yang sudah berjalan sebelum atur ulang (perangkat lain / penyerang dengan sesi curian).
        lama = sesi_aktif(realm, user)
        periksa(lama is not None, f"[{realm}] sesi aktif sebelum atur ulang tersedia (password + OTP)")
        sid_lama = klaim(lama["access_token"]).get("sid") if lama else None
        password_lama = user.password
        sebelum = daftar_email()
        b, _, r = mulai(realm)
        tautan = [h for _, h in k.Page(r.body).links if "reset-credentials" in (h or "")]
        periksa(bool(tautan), f"[{realm}] tautan lupa password ada di halaman login")
        if not tautan:
            return
        r = ikuti(b, b.open(tautan[0]))
        f = k.Page(r.body).form("kc-reset-password-form")
        r = ikuti(b, b.open(f["action"], dict(f["inputs"], username=user.email)))
        nama, raw = baca_email(sebelum)
        link = tautan_atur_ulang(raw) if raw else None
        periksa(link is not None, f"[{realm}] email atur ulang terkirim lewat SMTP realm dan memuat tautan token aksi")
        if not link:
            return
        b2 = k.Browser(BASE)
        r = ikuti(b2, b2.open(link))
        p = k.Page(r.body)
        periksa(p.form("kc-otp-login-form") is not None and p.form("kc-passwd-update-form") is None,
                f"[{realm}] tautan atur ulang membuka formulir OTP, bukan formulir password baru")
        sebelum_gagal = percobaan_gagal(realm, user.id).get("numSecondaryAuthFailures", 0)
        r = kirim_otp(b2, r, user.kode_salah())
        p = k.Page(r.body)
        periksa(p.form("kc-passwd-update-form") is None and kode_dari(r, realm) is None,
                f"[{realm}] OTP salah di alur atur ulang tidak membuka formulir password baru")
        sesudah = percobaan_gagal(realm, user.id)
        periksa(sesudah.get("numSecondaryAuthFailures", 0) == sebelum_gagal + 1 and sesudah.get("numFailures", 0) >= 1,
                f"[{realm}] OTP salah di alur atur ulang dihitung penguncian (F1-T74)", str(sesudah))
        r = kirim_otp(b2, r, user.otp())
        p = k.Page(r.body)
        f = p.form("kc-passwd-update-form")
        periksa(f is not None, f"[{realm}] OTP benar baru membuka formulir password baru")
        if f:
            # TD-27: formulir atur ulang selalu mengirim logout-sessions=on, tanpa kotak pilihan (bawaan keycloak.v2
            # tidak tercentang); UpdatePassword 26.8.2 lalu mengakhiri semua sesi lain, online dan offline (SRC-770).
            periksa(f["inputs"].get("logout-sessions") == "on" and 'type="checkbox"' not in r.body
                    and 'id="rs-logout-info"' in r.body,
                    f"[{realm}] formulir password baru mengirim logout-sessions=on tanpa pilihan dan memberi tahu semua sesi diakhiri",
                    str(f["inputs"].get("logout-sessions")))
            baru = "Baru-" + secrets.token_urlsafe(12)
            mulai_ms = int(time.time() * 1000) - 5000
            email_sebelum_simpan = daftar_email()
            time.sleep(JEDA)
            r = ikuti(b2, b2.open(f["action"], dict(f["inputs"], **{"password-new": baru, "password-confirm": baru})))
            user.password = baru
            if lama:
                rs_ = segarkan(realm, lama["refresh_token"])
                periksa(rs_.status == 400 and (rs_.json() or {}).get("error") == "invalid_grant",
                        f"[{realm}] token penyegar sesi lama ditolak seketika setelah atur ulang: sesi Keycloak berakhir (US-103 AC4d)",
                        f"HTTP {rs_.status}")
            ev = ADMIN.get(f"{realm}/events?user={user.id}&dateFrom={mulai_ms}&max=50")
            jenis = sorted({e.get("type") for e in ev})
            print(f"  INFO [{realm}] event Keycloak saat atur ulang: {jenis}")
            keluar = [e for e in ev if e.get("type") == "LOGOUT" and e.get("sessionId") == sid_lama
                      and (e.get("details") or {}).get("logout_triggered_by_required_action") == "UPDATE_PASSWORD"]
            periksa(sid_lama is not None and len(keluar) == 1 and keluar[0].get("ipAddress"),
                    f"[{realm}] event LOGOUT untuk sesi lama (sessionId = klaim sid, beralamat IP) tercatat: dasar pencabutan "
                    f"session_revocation oleh rs-backend worker (TD-27)", str(jenis))
            ganti = [e for e in ev if e.get("type") == "UPDATE_CREDENTIAL" and (e.get("details") or {}).get("credential_type") == "password"
                     and not e.get("error")]
            periksa(len(ganti) == 1, f"[{realm}] event UPDATE_CREDENTIAL password tercatat sekali (E-04 lewat pengambil event)", str(jenis))
            nama, raw = baca_email(email_sebelum_simpan)
            subjek, isi = teks_email(raw) if raw else ("", "")
            periksa(subjek == SUBJEK_PASSWORD_DIUBAH and "Layanan Pelanggan" in isi,
                    f"[{realm}] email pemberitahuan 'password diubah' terkirim ke email akun lewat SMTP realm (US-103 AC4d, KEPUTUSAN-200)",
                    subjek)
            if raw:
                periksa(not re.search(r"\b\d{1,3}(?:\.\d{1,3}){3}\b", isi) and "http" not in isi.lower()
                        and baru not in raw and password_lama not in raw and "action-token" not in raw,
                        f"[{realm}] email pemberitahuan tanpa alamat IP, tautan, atau password")
            periksa(kode_dari(r, realm) is None,
                    f"[{realm}] setelah password baru tersimpan, atur ulang TIDAK menerbitkan kode otorisasi (US-103 AC4d, TD-16)",
                    (r.location or "")[:80])
            _, _, rs = mulai(realm, browser=b2)
            periksa(kode_dari(rs, realm) is None and k.Page(rs.body).form("kc-form-login") is not None,
                    f"[{realm}] sesi peramban atur ulang tidak menjadi sesi login: permintaan otorisasi baru meminta login dari awal")
            b3, _, r3 = mulai(realm)
            r3 = kirim_password(b3, r3, user.email, baru)
            periksa(k.Page(r3.body).form("kc-otp-login-form") is not None and kode_dari(r3, realm) is None,
                    f"[{realm}] setelah atur ulang, login dengan password baru tetap meminta OTP")
            b4, _, r4 = mulai(realm)
            sebelum_gagal = daftar_email()
            r4 = kirim_password(b4, r4, user.email, password_lama)
            periksa(k.Page(r4.body).form("kc-otp-login-form") is None and kode_dari(r4, realm) is None
                    and pesan_galat(r4) == PESAN_SERAGAM,
                    f"[{realm}] password lama tidak dapat dipakai setelah atur ulang (pesan seragam)")
            periksa(tunggu_tanpa_email_baru(sebelum_gagal),
                    f"[{realm}] login gagal tidak mengirim email (listener email hanya UPDATE_PASSWORD, PRD §2.2)")
        r = ikuti(k.Browser(BASE), k.Browser(BASE).open(link))
        periksa(k.Page(r.body).form("kc-passwd-update-form") is None and k.Page(r.body).form("kc-otp-login-form") is None,
                f"[{realm}] tautan atur ulang yang sudah dipakai ditolak")
    finally:
        pulihkan_smtp(realm, asli)


def cek_grant_password(realm, user):
    r = k.request("POST", f"{BASE}/realms/{realm}/protocol/openid-connect/token",
                  form={"grant_type": "password", "client_id": "admin-cli", "username": user.email, "password": user.password})
    periksa(r.status in (400, 401) and "access_token" not in (r.json() or {}), f"[{realm}] grant password lewat admin-cli ditolak", f"HTTP {r.status}")


def smtp_ke_penangkap(realm):
    """Mode sink: SMTP realm sementara ke penangkap uji (tanpa TLS, 127.0.0.1 namespace Keycloak); mengembalikan
    konfigurasi asli untuk dipulihkan dengan pulihkan_smtp(). Mode relay: konfigurasi realm tidak diubah (email
    lewat relay ADR-026 sungguhan); mengembalikan None."""
    if EMAIL_MODE == "relay":
        return None
    asli = ADMIN.get(realm).get("smtpServer") or {}
    ADMIN.call("PUT", realm, {"smtpServer": dict(asli, host="127.0.0.1", port=SINK_PORT, starttls="false")})
    return asli


def pulihkan_smtp(realm, asli):
    if asli is None:
        return
    ADMIN.call("PUT", realm, {"smtpServer": asli})
    periksa((ADMIN.get(realm).get("smtpServer") or {}) == asli, f"[{realm}] SMTP realm dikembalikan ke konfigurasi relay setelah fixture")


def cek_verifikasi_email_dulu(realm):
    print(f"== [{realm}] verifikasi email sebelum pendaftaran TOTP (pre-account hijack)")
    user = UserUji(realm, "belumverif", email_terverifikasi=False)
    asli = smtp_ke_penangkap(realm)
    try:
        b, _, r = mulai(realm)
        p = k.Page(r.body)
        time.sleep(JEDA)
        r = b.open(p.form("kc-form-login")["action"], {"username": user.email, "password": user.password, "credentialId": ""})
        pertama = r.location or ""
        periksa("execution=VERIFY_EMAIL" in pertama, f"[{realm}] email belum terverifikasi: required action pertama Verify Email",
                pertama.split("?")[0] if pertama else f"HTTP {r.status}")
        r = ikuti(b, r)
        p = k.Page(r.body)
        periksa("template: login-verify-email.ftl" in r.body and p.form("kc-totp-settings-form") is None and kode_dari(r, realm) is None,
                f"[{realm}] halaman pertama setelah password adalah verifikasi email, tanpa formulir TOTP dan tanpa kode otorisasi")
        if "execution=VERIFY_EMAIL" in pertama:
            r = ikuti(b, b.open(pertama.replace("execution=VERIFY_EMAIL", "execution=CONFIGURE_TOTP")))
            periksa(k.Page(r.body).form("kc-totp-settings-form") is None and kode_dari(r, realm) is None,
                    f"[{realm}] membuka langsung Configure OTP sebelum email terverifikasi tidak menampilkan formulir TOTP")
        creds = [c["type"] for c in ADMIN.get(f"{realm}/users/{user.id}/credentials")]
        periksa("otp" not in creds, f"[{realm}] user dengan email belum terverifikasi tidak memiliki kredensial OTP", str(creds))
    finally:
        pulihkan_smtp(realm, asli)


# ------------------------------------------------------------------------------- relay email (F1-T36)


def docker_out(*args):
    return subprocess.run(["docker", *args], capture_output=True, text=True)


def ses_mode(status):
    """Jawaban tiruan SES berikutnya: 200 (sukses) atau kode galat HTTP."""
    r = docker_out("exec", SINK, "sh", "-c", f"echo {int(status)} > /tmp/ses-mode")
    if r.returncode != 0:
        raise RuntimeError("mode tiruan SES tidak dapat diubah")


def ses_json(nama_eml):
    """Isi JSON permintaan SendEmail yang menghasilkan <n>.eml."""
    n = nama_eml.split(".")[0]
    r = docker_out("exec", SINK, "cat", f"/tmp/ses-json/{n}.json")
    return json.loads(r.stdout) if r.returncode == 0 and r.stdout else None


def ses_tanda_tangan_buruk():
    r = docker_out("exec", SINK, "sh", "-c", "cat /tmp/ses-json/bad-signature 2>/dev/null | wc -l")
    return int((r.stdout or "0").strip() or 0)


def tunggu_tanpa_email_baru(sebelum, detik=3):
    batas = time.time() + detik
    while time.time() < batas:
        if [f for f in daftar_email() if f not in sebelum]:
            return False
        time.sleep(0.5)
    return True


def kirim_verifikasi(realm, user):
    return ADMIN.call("PUT", f"{realm}/users/{user.id}/send-verify-email")


def smtp_sementara(realm, **ubah):
    asli = ADMIN.get(realm).get("smtpServer") or {}
    ADMIN.call("PUT", realm, {"smtpServer": dict(asli, **ubah)})
    return asli


def pulihkan_smtp_relay(realm, asli):
    ADMIN.call("PUT", realm, {"smtpServer": asli})
    periksa((ADMIN.get(realm).get("smtpServer") or {}) == asli, f"[{realm}] SMTP realm dikembalikan ke relay setelah uji negatif")


def kirim_lupa_password(realm, email_):
    """Alur "lupa password" di halaman login; mengembalikan respons sesudah formulir dikirim."""
    b, _, r = mulai(realm)
    tautan = [h for _, h in k.Page(r.body).links if "reset-credentials" in (h or "")]
    if not tautan:
        raise RuntimeError("tautan lupa password tidak ditemukan")
    r = ikuti(b, b.open(tautan[0]))
    f = k.Page(r.body).form("kc-reset-password-form")
    return ikuti(b, b.open(f["action"], dict(f["inputs"], username=email_)))


def teks_halaman(r):
    """Teks halaman tanpa tag dan tanpa nilai yang berubah per permintaan (untuk membandingkan dua halaman)."""
    body = re.sub(r"<script.*?</script>", "", r.body or "", flags=re.S)
    body = re.sub(r"<[^>]+>", " ", body)
    body = re.sub(r"(session_code|execution|tab_id|client_data)=[^&\s\"']+", "", body)
    return " ".join(body.split())


def cek_relay_email(realm):
    """F1-T36 (ADR-026): email Keycloak lewat relay sungguhan ke tiruan SES, termasuk skenario negatif."""
    print(f"== [{realm}] email lewat relay ADR-026 (STARTTLS, SigV4, kegagalan SES, EMAIL_SEND_FAILED)")
    user = UserUji(realm, "relay", email_terverifikasi=False)
    dari = os.environ.get("RS_KC_SMTP_FROM")
    try:
        ses_mode(200)
        sebelum = daftar_email()
        r = kirim_verifikasi(realm, user)
        periksa(r.status == 204, f"[{realm}] Admin REST send-verify-email sukses lewat relay", f"HTTP {r.status}")
        nama, raw = baca_email(sebelum)
        periksa(raw is not None, f"[{realm}] email verifikasi sampai ke tiruan SES lewat relay (STARTTLS, CA uji)")
        if raw:
            m = email.message_from_string(raw)
            periksa(user.email in (m.get("To") or "") and dari in (m.get("From") or "")
                    and "login-actions/action-token" in raw,
                    f"[{realm}] MIME diteruskan apa adanya: To penerima, From pengirim realm, tautan token aksi di badan")
            j = ses_json(nama) or {}
            periksa(j.get("FromEmailAddress") == dari and j.get("Destination") == {"ToAddresses": [user.email]}
                    and j.get("ConfigurationSetName") == SES_CONFIG_SET and "EndpointId" not in j,
                    f"[{realm}] permintaan SES disusun relay: FromEmailAddress dari daftar izin, Destination dari RCPT, "
                    "ConfigurationSetName eksplisit, tanpa EndpointId (C-01)",
                    str({x: j.get(x) for x in ("FromEmailAddress", "ConfigurationSetName")}))
            periksa((j.get("ConfigurationOverrides") or {}).get("Tracking")
                    == {"ClickTrackingEnabled": "DISABLED", "OpenTrackingEnabled": "DISABLED"},
                    f"[{realm}] pelacakan klik/buka SES dimatikan per pesan (ADR-026 butir 5)")
        periksa(ses_tanda_tangan_buruk() == 0, f"[{realm}] tiruan SES memvalidasi SigV4 relay tanpa penolakan")

        # Jalur Admin REST: SES gagal -> relay 4xx/5xx -> Keycloak HTTP 500, tanpa email (ADR-026 butir 3).
        for status in (503, 429, 400):
            ses_mode(status)
            sebelum = daftar_email()
            r = kirim_verifikasi(realm, user)
            periksa(r.status == 500 and tunggu_tanpa_email_baru(sebelum, 1),
                    f"[{realm}] SES {status}: Admin REST send-verify-email gagal (HTTP 500), tanpa email", f"HTTP {r.status}")
            periksa(user.email not in (r.body or "") and "ref=" not in (r.body or ""),
                    f"[{realm}] galat Admin REST tidak menggemakan alamat atau balasan relay (SES {status})")

        # Alur "lupa password": halaman sama dengan saat sukses; kegagalan hanya di event (ADR-026 butir 8).
        ses_mode(200)
        halaman_sukses = teks_halaman(kirim_lupa_password(realm, user.email))
        ses_mode(503)
        mulai_t = int(time.time() * 1000) - 1000
        r = kirim_lupa_password(realm, user.email)
        periksa(teks_halaman(r) == halaman_sukses and halaman_sukses != "",
                f"[{realm}] lupa password saat SES gagal menampilkan halaman yang sama dengan saat sukses (SRC-364)")
        ev = []
        for _ in range(10):
            ev = [e for e in ADMIN.get(f"{realm}/events?type=SEND_RESET_PASSWORD_ERROR&user={user.id}&max=20")
                  if e.get("time", 0) >= mulai_t]
            if ev:
                break
            time.sleep(0.5)
        periksa(any(e.get("error") == "email_send_failed" for e in ev),
                f"[{realm}] event SEND_RESET_PASSWORD_ERROR dengan galat email_send_failed tercatat (ADR-026 butir 8)",
                str([e.get("error") for e in ev]))
        detail = json.dumps([e.get("details") or {} for e in ev])
        alasan = json.dumps([(e.get("details") or {}).get("reason", "") for e in ev])
        periksa(user.email not in alasan and "action-token" not in detail and "key=" not in detail and "ref=" not in alasan,
                f"[{realm}] detail alasan EMAIL_SEND_FAILED tanpa alamat, token aksi, atau balasan relay (M-35)", alasan)
        print(f"  OBSERVASI [{realm}] kunci detail event EMAIL_SEND_FAILED: "
              f"{sorted({x for e in ev for x in (e.get('details') or {})})}; alasan: {alasan}")

        # Required action Verify Email saat login ketika SES gagal (ADR-026 butir 3, sebelumnya [TIDAK DIKETAHUI]).
        user2 = UserUji(realm, "relaylogin", email_terverifikasi=False)
        sebelum = daftar_email()
        b, _, r = mulai(realm)
        r = kirim_password(b, r, user2.email, user2.password)
        verif = "template: login-verify-email.ftl" in (r.body or "")
        periksa(kode_dari(r, realm) is None and tunggu_tanpa_email_baru(sebelum, 1),
                f"[{realm}] login dengan email belum terverifikasi saat SES gagal: tanpa kode otorisasi dan tanpa email")
        galat = re.findall(r'(?:alert-error|kc-feedback-text|pf-m-danger)[^>]*>\s*([^<]{0,160})', r.body or "")
        print(f"  OBSERVASI [{realm}] Verify Email saat login dan SES gagal: HTTP {r.status}, halaman verifikasi email={verif}, "
              f"teks galat={galat[:2]}")

        # Relay menolak koneksi tanpa STARTTLS dan pengirim di luar daftar izin (Keycloak menerima galat).
        ses_mode(200)
        for ubah, nama_uji in (({"starttls": "false"}, "STARTTLS dimatikan di realm"),
                               ({"from": "penyusup@uji.invalid"}, "pengirim realm di luar daftar izin relay")):
            asli = smtp_sementara(realm, **ubah)
            try:
                sebelum = daftar_email()
                r = kirim_verifikasi(realm, user)
                periksa(r.status == 500 and tunggu_tanpa_email_baru(sebelum, 1),
                        f"[{realm}] {nama_uji}: relay menolak, Admin REST HTTP 500, tanpa email", f"HTTP {r.status}")
            finally:
                pulihkan_smtp_relay(realm, asli)

        # Log relay dan Keycloak: tanpa alamat, subjek, atau token (ADR-005, ADR-010, ADR-026 butir 4).
        log_relay = docker_out("logs", RELAY)
        teks_relay = (log_relay.stdout or "") + (log_relay.stderr or "")
        periksa('"outcome":"sent"' in teks_relay and '"outcome":"failed_temporary"' in teks_relay
                and "from_not_allowed" in teks_relay and "starttls_required" in teks_relay,
                f"[{realm}] log relay mencatat hasil (sent, failed_temporary) dan kelas penolakan")
        bocor = [x for x in (user.email, user2.email, dari, "penyusup", "action-token", "key=eyJ", "Subject")
                 if x and x in teks_relay]
        periksa(not bocor, f"[{realm}] log relay tanpa alamat, subjek, atau token aksi", str(bocor))
        log_kc = docker_out("logs", KC_CONTAINER)
        teks_kc = (log_kc.stdout or "") + (log_kc.stderr or "")
        periksa("action-token" not in teks_kc and "key=eyJ" not in teks_kc,
                f"[{realm}] log Keycloak tanpa token aksi saat pengiriman gagal")
    finally:
        ses_mode(200)


def cek_konsol_akun(realm, user):
    print(f"== [{realm}] konsol akun Keycloak tertutup")
    klien = {c["clientId"]: c for c in ADMIN.get(f"{realm}/clients")}
    periksa(klien.get("account-console", {}).get("enabled") is False, f"[{realm}] klien account-console dinonaktifkan")
    _, _, r = mulai(realm, pkce=True, klien={"client_id": "account-console", "redirect": f"{BASE}/realms/{realm}/account/"})
    periksa(k.Page(r.body).form("kc-form-login") is None and kode_dari(r, realm) is None,
            f"[{realm}] permintaan login ke account-console ditolak (tidak ada formulir login)", f"HTTP {r.status}")
    default = [c["name"] for c in ADMIN.get(f"{realm}/roles/default-roles-{realm}/composites")]
    periksa("manage-account" not in default and "manage-account-links" not in default,
            f"[{realm}] default role tanpa account/manage-account", str(default))
    akun = klien.get("account", {}).get("id")
    efektif = [x["name"] for x in ADMIN.get(f"{realm}/users/{user.id}/role-mappings/clients/{akun}/composite")] if akun else []
    periksa("manage-account" not in efektif, f"[{realm}] user baru tidak memiliki peran manage-account", str(efektif))


BUILTIN_CLIENTS = {"account", "account-console", "admin-cli", "broker", "realm-management", "security-admin-console"}


def cek_drift(realm):
    """Impor keycloak-config-cli hanya menimpa yang ada di JSON; klien/IdP tambahan buatan manual tidak
    terhapus, sehingga keberadaannya dideteksi di sini."""
    print(f"== [{realm}] deteksi drift (klien dan IdP di luar konfigurasi)")
    berkas = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "realms", f"{realm}.json")
    with open(berkas, encoding="utf-8") as fh:
        dikelola = {c["clientId"] for c in json.load(fh).get("clients", [])} | BUILTIN_CLIENTS
    ada = {c["clientId"] for c in ADMIN.get(f"{realm}/clients")}
    periksa(ada <= dikelola, f"[{realm}] tidak ada klien di luar konfigurasi", str(sorted(ada - dikelola)))
    idp = ADMIN.get(f"{realm}/identity-provider/instances")
    periksa(idp == [], f"[{realm}] tidak ada identity provider", str([i.get("alias") for i in idp]))


PEMBACA = "rs-backend-event-reader"


def cek_pembaca_event(realm):
    """F1-T34 (ADR-011 butir 10): klien layanan pengambil event berhak minimum, dan bentuk event yang
    dipetakan backend (crates rs-backend modul identity) sama dengan Keycloak sungguhan."""
    print(f"== [{realm}] klien pembaca event rs-backend worker (F1-T34)")
    klien = {c["clientId"]: c for c in ADMIN.get(f"{realm}/clients")}
    c = klien.get(PEMBACA)
    if not periksa(c is not None, f"[{realm}] klien {PEMBACA} ada"):
        return
    periksa(c.get("publicClient") is False and c.get("serviceAccountsEnabled") is True
            and c.get("standardFlowEnabled") is False and c.get("implicitFlowEnabled") is False
            and c.get("directAccessGrantsEnabled") is False and c.get("fullScopeAllowed") is False
            and not c.get("redirectUris"),
            f"[{realm}] {PEMBACA}: rahasia, hanya client credentials, tanpa redirect, fullScopeAllowed mati")
    cakupan = sorted(x["name"] for x in ADMIN.get(f"{realm}/clients/{c['id']}/default-client-scopes"))
    opsional = ADMIN.get(f"{realm}/clients/{c['id']}/optional-client-scopes")
    periksa("kc-admin-api" in cakupan and not ({"profile", "email", "address", "phone", "roles"} & set(cakupan)) and opsional == [],
            f"[{realm}] {PEMBACA}: cakupan kc-admin-api (audience Admin REST 26.8.2), tanpa cakupan data pribadi", str(cakupan))
    nama_env = f"RS_KC_EVENT_READER_{realm.upper()}_SECRET"
    rahasia = os.environ.get(nama_env)
    s = ADMIN.get(f"{realm}/clients/{c['id']}/client-secret")
    periksa(rahasia and s.get("value") == rahasia, f"[{realm}] {PEMBACA}: secret berasal dari variabel lingkungan saat penerapan")
    rm = ADMIN.get(f"{realm}/clients?clientId=realm-management")
    sa = ADMIN.get(f"{realm}/clients/{c['id']}/service-account-user")
    efektif = sorted(x["name"] for x in ADMIN.get(f"{realm}/users/{sa['id']}/role-mappings/clients/{rm[0]['id']}/composite"))
    periksa(efektif == ["view-events"], f"[{realm}] service account hanya realm-management/view-events", str(efektif))
    peta = sorted(x["name"] for x in ADMIN.get(f"{realm}/clients/{c['id']}/scope-mappings/clients/{rm[0]['id']}"))
    periksa(peta == ["view-events"], f"[{realm}] cakupan peran token klien hanya view-events", str(peta))

    r = k.request("POST", f"{BASE}/realms/{realm}/protocol/openid-connect/token",
                  form={"grant_type": "client_credentials", "client_id": PEMBACA, "client_secret": rahasia or ""})
    if not periksa(r.status == 200, f"[{realm}] token client credentials terbit", f"HTTP {r.status}"):
        return
    h = {"Authorization": "Bearer " + r.json()["access_token"]}

    def status(metode, path):
        return k.request(metode, f"{BASE}/admin/realms/{path}", headers=h).status

    awal = int(time.time() * 1000) - 300_000
    surel = f"t34-{secrets.token_hex(4)}@uji.invalid"
    b, _, r0 = mulai(realm)
    kirim_password(b, r0, surel, "Salah-" + secrets.token_hex(4))
    r = k.request("GET", f"{BASE}/admin/realms/{realm}/events?type=LOGIN_ERROR&direction=asc&first=0&max=500&dateFrom={awal}", headers=h)
    ev = [e for e in (r.json() or []) if (e.get("details") or {}).get("username") == surel] if r.status == 200 else []
    e0 = ev[0] if len(ev) == 1 else {}
    periksa(r.status == 200 and len(ev) == 1 and e0.get("error") == "user_not_found" and "userId" not in e0
            and isinstance(e0.get("time"), int) and len(e0.get("id", "")) == 36 and e0.get("ipAddress")
            and e0.get("clientId") == KLIEN[realm]["client_id"],
            f"[{realm}] pembaca event: LOGIN_ERROR email tak terdaftar berbentuk yang dipetakan backend "
            "(id UUID, time ms, error user_not_found, tanpa userId, details.username = email)", f"HTTP {r.status}, {len(ev)} event")
    lain = "admin" if realm == "pasien" else "pasien"
    hasil = {p: status(m, q) for p, (m, q) in {
        "users": ("GET", f"{realm}/users"), "clients": ("GET", f"{realm}/clients"),
        "hapus-event": ("DELETE", f"{realm}/events"), "event-realm-lain": ("GET", f"{lain}/events")}.items()}
    periksa(all(v == 403 for v in hasil.values()),
            f"[{realm}] pembaca event tidak dapat membaca user/klien, menghapus event, atau membaca realm lain", str(hasil))
    periksa(status("GET", f"{realm}/events?type=TIDAK_ADA") == 500,
            f"[{realm}] type event tak dikenal dijawab 500 (daftar type backend wajib persis nama EventType)")


PENGELOLA = "rs-backend-admin-manager"


def cek_pengelola_admin(realm):
    """F1-T65 (US-208 AC1, AC4; ADR-011): klien layanan pengelola pengguna admin berhak minimum, dan siklus yang
    dipakai backend (apps/rs-backend modul identity, keycloak.rs) berjalan di Keycloak sungguhan: buat pengguna
    tanpa kredensial (201 + Location), undangan execute-actions-email lifespan 86400 berisi tautan token aksi,
    aktivasi lewat tautan (verifikasi email, TOTP, password), tautan yang sudah dipakai ditolak, login wajib OTP,
    nonaktifkan + akhiri sesi, hapus."""
    print(f"== [{realm}] klien pengelola pengguna admin rs-backend (F1-T65)")
    klien = {c["clientId"]: c for c in ADMIN.get(f"{realm}/clients")}
    c = klien.get(PENGELOLA)
    if realm != "admin":
        periksa(c is None, f"[{realm}] klien {PENGELOLA} hanya ada di realm admin")
        return
    if not periksa(c is not None, f"[{realm}] klien {PENGELOLA} ada"):
        return
    periksa(c.get("publicClient") is False and c.get("serviceAccountsEnabled") is True
            and c.get("standardFlowEnabled") is False and c.get("implicitFlowEnabled") is False
            and c.get("directAccessGrantsEnabled") is False and c.get("fullScopeAllowed") is False
            and not c.get("redirectUris"),
            f"[{realm}] {PENGELOLA}: rahasia, hanya client credentials, tanpa redirect, fullScopeAllowed mati")
    cakupan = sorted(x["name"] for x in ADMIN.get(f"{realm}/clients/{c['id']}/default-client-scopes"))
    # `service_account` ditambahkan Keycloak otomatis untuk klien service account (klaim client_id, tanpa data pribadi).
    periksa(set(cakupan) - {"service_account"} == {"kc-admin-api"},
            f"[{realm}] {PENGELOLA}: hanya cakupan kc-admin-api (+ service_account bawaan)", str(cakupan))
    rahasia = os.environ.get("RS_KC_ADMIN_MANAGER_SECRET")
    s = ADMIN.get(f"{realm}/clients/{c['id']}/client-secret")
    periksa(rahasia and s.get("value") == rahasia, f"[{realm}] {PENGELOLA}: secret berasal dari variabel lingkungan saat penerapan")
    rm = ADMIN.get(f"{realm}/clients?clientId=realm-management")
    sa = ADMIN.get(f"{realm}/clients/{c['id']}/service-account-user")
    efektif = sorted(x["name"] for x in ADMIN.get(f"{realm}/users/{sa['id']}/role-mappings/clients/{rm[0]['id']}/composite"))
    # manage-users di Keycloak 26.8.2 tidak komposit; query-* boleh muncul bila Keycloak menambahkannya.
    periksa(efektif == ["manage-users"], f"[{realm}] service account hanya realm-management/manage-users", str(efektif))
    realm_roles = sorted(x["name"] for x in ADMIN.get(f"{realm}/users/{sa['id']}/role-mappings/realm"))
    peta = sorted(x["name"] for x in ADMIN.get(f"{realm}/clients/{c['id']}/scope-mappings/clients/{rm[0]['id']}"))
    periksa(peta == ["manage-users"], f"[{realm}] cakupan peran token klien hanya manage-users", str(peta))

    r = k.request("POST", f"{BASE}/realms/{realm}/protocol/openid-connect/token",
                  form={"grant_type": "client_credentials", "client_id": PENGELOLA, "client_secret": rahasia or ""})
    if not periksa(r.status == 200, f"[{realm}] token client credentials pengelola terbit", f"HTTP {r.status}"):
        return
    h = {"Authorization": "Bearer " + r.json()["access_token"]}

    def panggil(metode, path, body=None):
        return k.request(metode, f"{BASE}/admin/realms/{path}", headers=h, data=body)

    asli = smtp_ke_penangkap(realm)
    uid = None
    try:
        surel = f"t65-{secrets.token_hex(4)}@uji.invalid"
        r = panggil("POST", f"{realm}/users", {"username": surel, "email": surel, "enabled": True, "emailVerified": False})
        lokasi = r.location or ""
        periksa(r.status == 201 and "/users/" in lokasi, f"[{realm}] pengelola membuat pengguna admin (201 + Location)", f"HTTP {r.status}")
        if r.status != 201:
            return
        uid = lokasi.rsplit("/", 1)[1]
        USER_UJI.append((realm, uid))
        r = panggil("POST", f"{realm}/users", {"username": surel, "email": surel, "enabled": True})
        periksa(r.status == 409, f"[{realm}] email yang sama ditolak 409 (dipetakan backend ke already_registered)", f"HTTP {r.status}")
        creds = ADMIN.get(f"{realm}/users/{uid}/credentials")
        periksa(creds == [], f"[{realm}] pengguna baru tanpa kredensial (tidak dapat masuk sebelum aktivasi)", str(creds))

        # Hak minimum: tidak dapat membaca klien, memetakan peran realm-management, impersonasi, atau realm lain.
        manage_realm = [x for x in ADMIN.get(f"{realm}/clients/{rm[0]['id']}/roles") if x["name"] == "manage-realm"]
        hasil = {
            "baca-klien": panggil("GET", f"{realm}/clients").status,
            "peta-manage-realm": panggil("POST", f"{realm}/users/{uid}/role-mappings/clients/{rm[0]['id']}", manage_realm).status,
            "peta-manage-users-ke-diri": panggil("POST", f"{realm}/users/{sa['id']}/role-mappings/clients/{rm[0]['id']}",
                                                 [x for x in ADMIN.get(f"{realm}/clients/{rm[0]['id']}/roles") if x["name"] == "manage-clients"]).status,
            "impersonasi": panggil("POST", f"{realm}/users/{uid}/impersonation").status,
            "realm-pasien": panggil("GET", "pasien/users").status,
            "baca-event": panggil("GET", f"{realm}/events").status,
        }
        periksa(all(v == 403 for v in hasil.values()),
                f"[{realm}] pengelola tidak dapat membaca klien, memetakan peran admin Keycloak, impersonasi, membaca event, atau realm lain",
                str(hasil))
        periksa(realm_roles == ["default-roles-admin"] or realm_roles == [],
                f"[{realm}] service account tanpa peran realm tambahan", str(realm_roles))

        sebelum = daftar_email()
        r = panggil("PUT", f"{realm}/users/{uid}/execute-actions-email?lifespan=86400",
                    ["VERIFY_EMAIL", "UPDATE_PASSWORD", "CONFIGURE_TOTP"])
        periksa(r.status == 204, f"[{realm}] undangan execute-actions-email lifespan 86400 diterima (204)", f"HTTP {r.status}")
        nama, raw = baca_email(sebelum)
        link = tautan_atur_ulang(raw) if raw else None
        periksa(link is not None, f"[{realm}] email undangan terkirim dan memuat tautan token aksi")
        if raw:
            teks = raw.replace("=\n", "")
            periksa("1 day" in teks or "1440" in teks or "24 jam" in teks or "1 hari" in teks or "24" in teks,
                    f"[{realm}] email undangan menyebut masa berlaku 24 jam")
        if not link:
            return
        user = UserUji.__new__(UserUji)
        user.realm, user.email, user.otp_key, user.recovery, user._dipakai = realm, surel, None, [], set()
        user.password = "Aktivasi-" + secrets.token_urlsafe(12)
        b = k.Browser(BASE)
        r = ikuti(b, b.open(link))
        for _ in range(8):
            p = k.Page(r.body)
            tautan = [hh for _, hh in p.links if "action-token" in (hh or "")]
            if p.form("kc-totp-settings-form"):
                r = daftar_mfa(b, r, user)
            elif p.form("kc-recovery-codes-settings-form"):
                r = simpan_kode_pemulihan(b, r, user)
            elif p.form("kc-passwd-update-form"):
                f = p.form("kc-passwd-update-form")
                time.sleep(JEDA)
                r = ikuti(b, b.open(f["action"], dict(f["inputs"], **{"password-new": user.password, "password-confirm": user.password})))
            elif tautan:
                r = ikuti(b, b.open(html.unescape(tautan[0])))
            else:
                break
        info = ADMIN.get(f"{realm}/users/{uid}")
        jenis = sorted(x["type"] for x in ADMIN.get(f"{realm}/users/{uid}/credentials"))
        periksa(info.get("emailVerified") is True and "otp" in jenis and "password" in jenis and not info.get("requiredActions"),
                f"[{realm}] aktivasi lewat tautan: email terverifikasi, password dan TOTP terdaftar, tanpa required action tersisa",
                f"{jenis} {info.get('requiredActions')} emailVerified={info.get('emailVerified')}")
        r = ikuti(k.Browser(BASE), k.Browser(BASE).open(link))
        p = k.Page(r.body)
        periksa(p.form("kc-passwd-update-form") is None and p.form("kc-totp-settings-form") is None
                and not [hh for _, hh in p.links if "action-token" in (hh or "")],
                f"[{realm}] tautan undangan yang sudah dipakai ditolak (sekali pakai)")
        if "otp" in jenis:
            b3, verifier, r3 = mulai(realm)
            r3 = kirim_password(b3, r3, surel, user.password)
            periksa(k.Page(r3.body).form("kc-otp-login-form") is not None and kode_dari(r3, realm) is None,
                    f"[{realm}] admin yang baru aktif wajib OTP saat masuk")
            r3 = kirim_otp(b3, r3, user.otp())
            periksa(kode_dari(r3, realm) is not None, f"[{realm}] admin yang baru aktif dapat masuk dengan password + OTP")

        r = panggil("PUT", f"{realm}/users/{uid}", {"enabled": False})
        periksa(r.status == 204 and ADMIN.get(f"{realm}/users/{uid}").get("enabled") is False,
                f"[{realm}] pengelola menonaktifkan pengguna (204)", f"HTTP {r.status}")
        r = panggil("POST", f"{realm}/users/{uid}/logout")
        periksa(r.status == 204 and ADMIN.get(f"{realm}/users/{uid}/sessions") == [],
                f"[{realm}] pengelola mengakhiri semua sesi pengguna (204)", f"HTTP {r.status}")
        if "otp" in jenis:
            b4, _, r4 = mulai(realm)
            r4 = kirim_password(b4, r4, surel, user.password)
            periksa(kode_dari(r4, realm) is None and k.Page(r4.body).form("kc-otp-login-form") is None,
                    f"[{realm}] pengguna nonaktif tidak dapat masuk")
        r = panggil("DELETE", f"{realm}/users/{uid}")
        periksa(r.status == 204, f"[{realm}] pengelola menghapus pengguna (kompensasi, 204)", f"HTTP {r.status}")
        r = panggil("DELETE", f"{realm}/users/{uid}")
        periksa(r.status == 404, f"[{realm}] hapus ulang dijawab 404 (backend menganggap sukses)", f"HTTP {r.status}")
    finally:
        pulihkan_smtp(realm, asli)


def cek_pasca_kunci():
    """US-103 AC3 (revisi, KEPUTUSAN-285): perilaku penguncian Keycloak 26.8.2 SETELAH masa kunci berakhir
    (SRC-525). Hitungan gagal hanya kembali nol setelah login berhasil atau Failure Reset Time 12 jam, sehingga
    satu gagal setelah kunci berakhir langsung mengunci lagi. Diuji di realm uji sementara dengan Wait
    Increment/Max Wait pendek; parameter lain disalin dari realm `pasien`."""
    print("== [uji] penguncian pasca-kunci (US-103 AC3, KEPUTUSAN-285; realm uji sementara)")
    nama = f"t31-kunci-{secrets.token_hex(3)}"
    sumber = ADMIN.get("pasien")
    tunggu = 4
    rep = {x: sumber[x] for x in ("bruteForceProtected", "permanentLockout", "maxTemporaryLockouts", "bruteForceStrategy",
                                  "failureFactor", "maxDeltaTimeSeconds", "quickLoginCheckMilliSeconds", "minimumQuickLoginWaitSeconds")}
    rep.update({"realm": nama, "enabled": True, "waitIncrementSeconds": tunggu, "maxFailureWaitSeconds": tunggu})
    r = ADMIN.call("POST", "", rep)
    if not periksa(r.status == 201, "[uji] realm uji pasca-kunci dibuat", f"HTTP {r.status}"):
        return
    try:
        redirect = "http://127.0.0.1/uji-callback"
        ADMIN.call("POST", f"{nama}/clients", {"clientId": "uji", "publicClient": True, "standardFlowEnabled": True,
                                                "directAccessGrantsEnabled": False, "redirectUris": [redirect]})
        # Profil bawaan realm baru mewajibkan nama depan/belakang (Verify Profile); tidak relevan di sini.
        vp = ADMIN.get(f"{nama}/authentication/required-actions/VERIFY_PROFILE")
        ADMIN.call("PUT", f"{nama}/authentication/required-actions/VERIFY_PROFILE", dict(vp, enabled=False))
        kl = {"client_id": "uji", "redirect": redirect}
        user = UserUji(nama, "pascakunci")

        def coba(password):
            b, _, r = mulai(nama, klien=kl)
            r = kirim_password(b, r, user.email, password)
            return r.status in (302, 303) and (r.location or "").startswith(redirect) and "code=" in r.location

        def status():
            return ADMIN.get(f"{nama}/attack-detection/brute-force/users/{user.id}")

        for _ in range(5):
            coba("Salah-" + secrets.token_hex(4))
        periksa(status().get("disabled") is True, "[uji] 5 gagal mengunci akun")
        time.sleep(tunggu + 1.5)
        periksa(status().get("disabled") is False, "[uji] kunci berakhir setelah Max Wait")
        coba("Salah-" + secrets.token_hex(4))
        st = status()
        periksa(st.get("disabled") is True and st.get("numFailures") == 6,
                "[uji] satu gagal setelah kunci berakhir langsung mengunci lagi; hitungan tidak direset (KEPUTUSAN-285)", str(st))
        time.sleep(tunggu + 1.5)
        periksa(coba(user.password) and status().get("numFailures") == 0,
                "[uji] login berhasil setelah kunci berakhir mereset hitungan ke 0")
        coba("Salah-" + secrets.token_hex(4))
        periksa(status().get("disabled") is False, "[uji] setelah hitungan direset, satu gagal tidak mengunci")
    finally:
        ADMIN.call("DELETE", nama)


def jalankan(nama, fungsi, *args):
    """Satu skenario; galat tak terduga dihitung GAGAL tanpa menghentikan skenario lain."""
    try:
        return fungsi(*args)
    except Exception as e:  # noqa: BLE001 (test: semua galat harus terlihat sebagai GAGAL)
        periksa(False, f"{nama} berjalan sampai selesai", f"{type(e).__name__}: {e}")
        return None


def main():
    r = k.request("GET", BASE + "/realms/master")
    if r.status != 200:
        print(f"Keycloak tidak siap di {BASE}: HTTP {r.status}", file=sys.stderr)
        return 1
    try:
        for realm in ("pasien", "admin"):
            r = k.request("GET", f"{BASE}/realms/{realm}")
            if not periksa(r.status == 200, f"[{realm}] realm ada"):
                continue
            jalankan(f"[{realm}] konfigurasi", cek_konfigurasi, realm)
            jalankan(f"[{realm}] drift", cek_drift, realm)
            jalankan(f"[{realm}] pembaca event", cek_pembaca_event, realm)
            jalankan(f"[{realm}] pengelola pengguna admin", cek_pengelola_admin, realm)
            print(f"== [{realm}] pendaftaran, PKCE")
            jalankan(f"[{realm}] pendaftaran mati", cek_pendaftaran_mati, realm)
            jalankan(f"[{realm}] PKCE wajib", cek_pkce_wajib, realm)
            user = jalankan(f"[{realm}] MFA dan token", cek_mfa_dan_token, realm)
            if user is not None:
                jalankan(f"[{realm}] grant password", cek_grant_password, realm, user)
                jalankan(f"[{realm}] konsol akun", cek_konsol_akun, realm, user)
                jalankan(f"[{realm}] kode pemulihan", cek_kode_pemulihan, realm, user)
                jalankan(f"[{realm}] atur ulang password", cek_atur_ulang_tidak_melewati_otp, realm, user)
            else:
                periksa(False, f"[{realm}] skenario kode pemulihan dan atur ulang password dijalankan",
                        "butuh user MFA dari skenario sebelumnya")
            jalankan(f"[{realm}] verifikasi email dulu", cek_verifikasi_email_dulu, realm)
            if EMAIL_MODE == "relay":
                jalankan(f"[{realm}] relay email", cek_relay_email, realm)
            else:
                print(f"== [{realm}] relay email end-to-end: hanya mode ephemeral (mode ini memakai penangkap SMTP)")
            jalankan(f"[{realm}] Argon2 dan kebijakan password", cek_argon2_dan_kebijakan, realm)
            jalankan(f"[{realm}] tema login", cek_tema, realm)
            jalankan(f"[{realm}] tema MFA", cek_halaman_mfa_tema, realm)
            jalankan(f"[{realm}] penguncian dan pesan", cek_penguncian_dan_pesan, realm)
            jalankan(f"[{realm}] OTP salah dikunci", cek_otp_dan_kode_pemulihan_salah_dikunci, realm)
            jalankan(f"[{realm}] batal pendaftaran TOTP", cek_batal_pendaftaran_totp, realm)
        jalankan("[uji] pasca-kunci", cek_pasca_kunci)
    finally:
        for realm, uid in USER_UJI:
            ADMIN.call("DELETE", f"{realm}/users/{uid}")
    print(f"keycloak-check: {LULUS} lulus, {GAGAL} gagal")
    return 0 if GAGAL == 0 and LULUS > 0 else 1


if __name__ == "__main__":
    sys.exit(main())
