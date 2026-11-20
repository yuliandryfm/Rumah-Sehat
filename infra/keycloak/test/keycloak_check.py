"""Test realm `pasien` dan `admin` sebagai kode terhadap Keycloak sungguhan (F1-T31, modul kritis T1).

Dijalankan oleh scripts/keycloak-check.sh setelah konfigurasi diterapkan dengan keycloak-config-cli.
Hanya pustaka standar Python 3. Semua user uji sintetis (domain .invalid) dan dihapus di akhir.

Dua kelompok pemeriksaan:
- konfigurasi lewat Admin REST (nilai realm, alur, required action, klien, profil user);
- perilaku lewat OIDC/halaman login: MFA wajib, kode pemulihan wajib, penguncian 5 gagal, pesan seragam,
  pendaftaran mati, PKCE wajib, rotasi token penyegar, Argon2, kebijakan password, atur ulang password
  tidak melewati OTP (US-103 AC4e), grant password ditolak.

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


def cek_kode_pemulihan(realm, user):
    print(f"== [{realm}] masuk dengan kode pemulihan")
    flows = flow_exec(realm, "rs-browser-mfa")
    rc_exec = next(e["id"] for e in flows if e.get("providerId") == "auth-recovery-authn-code-form")

    def pilih_kode_pemulihan():
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
    r = daftar_mfa(b, r, user)
    periksa(k.Page(r.body).form("kc-recovery-codes-settings-form") is not None and "Kode pemulihan hanya ditampilkan sekali" in r.body,
            f"[{realm}] halaman kode pemulihan bertema dan berbahasa Indonesia")
    periksa(not url_luar(r.body), f"[{realm}] halaman kode pemulihan tanpa URL ke domain luar", "; ".join(url_luar(r.body)[:5]))
    simpan_kode_pemulihan(b, r, user)
    b, _, r = mulai(realm)
    r = kirim_password(b, r, user.email, user.password)
    periksa(k.Page(r.body).form("kc-otp-login-form") is not None and "Kode OTP" in r.body and "Pakai cara lain" in r.body and 'lang="id"' in r.body,
            f"[{realm}] halaman OTP berbahasa Indonesia dengan tautan kode pemulihan")
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


def cek_atur_ulang_tidak_melewati_otp(realm, user):
    print(f"== [{realm}] atur ulang password tidak melewati OTP (US-103 AC4e)")
    # Konfigurasi relay produksi tetap diperiksa di cek_konfigurasi.
    asli = smtp_ke_penangkap(realm)
    try:
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
        r = kirim_otp(b2, r, user.kode_salah())
        p = k.Page(r.body)
        periksa(p.form("kc-passwd-update-form") is None and kode_dari(r, realm) is None,
                f"[{realm}] OTP salah di alur atur ulang tidak membuka formulir password baru")
        r = kirim_otp(b2, r, user.otp())
        p = k.Page(r.body)
        f = p.form("kc-passwd-update-form")
        periksa(f is not None, f"[{realm}] OTP benar baru membuka formulir password baru")
        if f:
            baru = "Baru-" + secrets.token_urlsafe(12)
            time.sleep(JEDA)
            r = ikuti(b2, b2.open(f["action"], dict(f["inputs"], **{"password-new": baru, "password-confirm": baru})))
            user.password = baru
            b3, _, r3 = mulai(realm)
            r3 = kirim_password(b3, r3, user.email, baru)
            periksa(k.Page(r3.body).form("kc-otp-login-form") is not None and kode_dari(r3, realm) is None,
                    f"[{realm}] setelah atur ulang, login dengan password baru tetap meminta OTP")
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
        jalankan("[uji] pasca-kunci", cek_pasca_kunci)
    finally:
        for realm, uid in USER_UJI:
            ADMIN.call("DELETE", f"{realm}/users/{uid}")
    print(f"keycloak-check: {LULUS} lulus, {GAGAL} gagal")
    return 0 if GAGAL == 0 and LULUS > 0 else 1


if __name__ == "__main__":
    sys.exit(main())
