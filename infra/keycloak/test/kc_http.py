"""Klien HTTP minimal untuk test realm Keycloak (F1-T31). Hanya pustaka standar Python 3.

Dipakai oleh keycloak_check.py. Hanya untuk Keycloak lokal/ephemeral di 127.0.0.1: penyimpan cookie
sengaja mengabaikan atribut Secure, karena Keycloak mode dev melayani HTTP tetapi menandai cookie
sesinya Secure (tanpa itu login browser tidak dapat diuji lewat HTTP lokal).
"""

import base64
import hashlib
import hmac
import html
import json
import re
import secrets
import struct
import time
import urllib.error
import urllib.parse
import urllib.request
from html.parser import HTMLParser


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):  # noqa: D401 (API urllib)
        return None


_OPENER = urllib.request.build_opener(_NoRedirect)


class Response:
    def __init__(self, status, headers, body):
        self.status = status
        self.headers = headers
        self.body = body

    @property
    def location(self):
        return self.headers.get("Location")

    def json(self):
        return json.loads(self.body) if self.body else None


def request(method, url, data=None, headers=None, form=None, timeout=30):
    body = None
    hdrs = dict(headers or {})
    if form is not None:
        body = urllib.parse.urlencode(form).encode()
        hdrs.setdefault("Content-Type", "application/x-www-form-urlencoded")
    elif data is not None:
        body = json.dumps(data).encode()
        hdrs.setdefault("Content-Type", "application/json")
    req = urllib.request.Request(url, data=body, method=method, headers=hdrs)
    try:
        r = _OPENER.open(req, timeout=timeout)
        return Response(r.status, r.headers, r.read().decode("utf-8", "replace"))
    except urllib.error.HTTPError as e:
        return Response(e.code, e.headers, e.read().decode("utf-8", "replace"))


class Browser:
    """Sesi browser sederhana: cookie per path, tanpa mengikuti redirect otomatis."""

    def __init__(self, base):
        self.base = base
        self.cookies = {}  # (path, name) -> value

    def _cookie_header(self, url):
        path = urllib.parse.urlsplit(url).path or "/"
        pairs = [f"{n}={v}" for (p, n), v in self.cookies.items() if path.startswith(p)]
        return "; ".join(pairs)

    def _store(self, headers):
        for raw in headers.get_all("Set-Cookie") or []:
            parts = [p.strip() for p in raw.split(";")]
            name, _, value = parts[0].partition("=")
            path = "/"
            expired = False
            for attr in parts[1:]:
                k, _, v = attr.partition("=")
                if k.lower() == "path" and v:
                    path = v
                elif k.lower() == "max-age" and v.strip() in ("0", "-1"):
                    expired = True
            if expired or value == "":
                self.cookies.pop((path, name), None)
            else:
                self.cookies[(path, name)] = value

    def open(self, url, form=None):
        url = urllib.parse.urljoin(self.base, html.unescape(url))
        headers = {}
        cookie = self._cookie_header(url)
        if cookie:
            headers["Cookie"] = cookie
        r = request("POST" if form is not None else "GET", url, form=form, headers=headers)
        self._store(r.headers)
        return r


class _FormParser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.forms = []
        self._cur = None
        self.links = []
        self._ids = {}
        self._stack = []
        self.texts_by_id = {}

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "form":
            self._cur = {"id": a.get("id"), "action": a.get("action"), "inputs": {}}
            self.forms.append(self._cur)
        elif tag in ("input", "textarea") and self._cur is not None:
            name = a.get("name")
            if name:
                typ = (a.get("type") or "text").lower()
                if typ in ("checkbox", "radio") and "checked" not in a:
                    self._cur["inputs"].setdefault(name, None)
                else:
                    self._cur["inputs"][name] = a.get("value", "")
        elif tag == "a" and a.get("href"):
            self.links.append((a.get("id"), a.get("href")))
        if tag not in ("input", "br", "img", "meta", "link", "hr"):
            self._stack.append(a.get("id"))
            if a.get("id"):
                self.texts_by_id.setdefault(a.get("id"), "")

    def handle_endtag(self, tag):
        if tag == "form":
            self._cur = None
        if tag not in ("input", "br", "img", "meta", "link", "hr") and self._stack:
            self._stack.pop()

    def handle_data(self, data):
        for ident in self._stack:
            if ident:
                self.texts_by_id[ident] = self.texts_by_id.get(ident, "") + data


class Page:
    def __init__(self, body):
        p = _FormParser()
        p.feed(body)
        self.body = body
        self.forms = p.forms
        self.links = p.links
        self.texts_by_id = {k: re.sub(r"\s+", " ", v).strip() for k, v in p.texts_by_id.items()}

    def form(self, form_id):
        for f in self.forms:
            if f["id"] == form_id:
                return f
        return None

    def form_ids(self):
        return [f["id"] for f in self.forms]

    def text(self, ident):
        return self.texts_by_id.get(ident)


def pkce_pair():
    verifier = secrets.token_urlsafe(48)
    challenge = base64.urlsafe_b64encode(hashlib.sha256(verifier.encode()).digest()).rstrip(b"=").decode()
    return verifier, challenge


def totp(key, period=30, digits=6, at=None, offset=0):
    """RFC 6238 TOTP dengan HMAC-SHA1 (kebijakan OTP realm, KEPUTUSAN-85).

    `key` adalah byte rahasia. Formulir Configure OTP Keycloak membawa rahasia mentah di field
    tersembunyi `totpSecret` (teks, dipakai apa adanya sebagai kunci HMAC), bukan bentuk base32.
    """
    if isinstance(key, str):
        key = key.encode()
    counter = int((time.time() if at is None else at) // period) + offset
    mac = hmac.new(key, struct.pack(">Q", counter), hashlib.sha1).digest()
    o = mac[-1] & 0x0F
    code = (struct.unpack(">I", mac[o : o + 4])[0] & 0x7FFFFFFF) % (10**digits)
    return str(code).zfill(digits)


def unescape(s):
    return html.unescape(s) if s else s
