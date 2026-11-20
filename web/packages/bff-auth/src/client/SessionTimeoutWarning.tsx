"use client";

// Dialog peringatan batas sesi web (KEPUTUSAN-170, design guideline §5.5, wireframe C01 layar 2):
// - batas tidak aktif 30 menit: peringatan 2 menit sebelum berakhir dengan "Tetap masuk" (SC 2.2.1 opsi
//   "Extend") dan "Keluar sekarang";
// - batas mutlak 12 jam: peringatan tanpa perpanjangan.
// Sumber kebenaran batas adalah server (`GET /auth/session`, tidak dihitung sebagai aktivitas). Komponen ini
// hanya menerima waktu batas dan token CSRF; token OIDC tidak pernah sampai ke browser.

import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "@rs/ui";

export interface SessionTimeoutWarningProps {
  readonly sessionUrl?: string;
  readonly extendUrl?: string;
  readonly logoutUrl?: string;
  readonly loginUrl?: string;
  /** Selang pemeriksaan status sesi (ms). Bawaan 30 detik. */
  readonly pollIntervalMs?: number;
  /** Dipanggil saat sesi berakhir; bawaan: pindah ke halaman masuk dengan `return_to` halaman ini. */
  readonly onExpired?: () => void;
}

interface Status {
  readonly idleExpiresAt: number;
  readonly absoluteExpiresAt: number;
  /** Selisih jam server terhadap jam browser (ms). */
  readonly skewMs: number;
  readonly warningMs: number;
  readonly csrfToken: string;
}

interface StatusBody {
  readonly data?: {
    readonly idle_expires_at?: unknown;
    readonly absolute_expires_at?: unknown;
    readonly server_time?: unknown;
    readonly warning_seconds?: unknown;
    readonly csrf_token?: unknown;
  };
}

function parseStatus(body: StatusBody, receivedAt: number): Status | null {
  const d = body.data;
  if (
    d === undefined ||
    typeof d.idle_expires_at !== "string" ||
    typeof d.absolute_expires_at !== "string" ||
    typeof d.server_time !== "string" ||
    typeof d.warning_seconds !== "number" ||
    typeof d.csrf_token !== "string"
  ) {
    return null;
  }
  return {
    idleExpiresAt: Date.parse(d.idle_expires_at),
    absoluteExpiresAt: Date.parse(d.absolute_expires_at),
    skewMs: Date.parse(d.server_time) - receivedAt,
    warningMs: d.warning_seconds * 1000,
    csrfToken: d.csrf_token,
  };
}

function formatSisa(ms: number): string {
  const total = Math.max(0, Math.ceil(ms / 1000));
  return `${String(Math.floor(total / 60))}:${String(total % 60).padStart(2, "0")}`;
}

export function SessionTimeoutWarning({
  sessionUrl = "/auth/session",
  extendUrl = "/auth/session/extend",
  logoutUrl = "/auth/logout",
  loginUrl = "/auth/login",
  pollIntervalMs = 30_000,
  onExpired,
}: SessionTimeoutWarningProps) {
  const [status, setStatus] = useState<Status | null>(null);
  const [now, setNow] = useState(() => Date.now());
  const [extending, setExtending] = useState(false);
  const [extendFailed, setExtendFailed] = useState(false);
  const [absoluteDismissed, setAbsoluteDismissed] = useState(false);
  const dialogRef = useRef<HTMLDivElement>(null);
  const expiredRef = useRef(false);

  const expire = useCallback(() => {
    if (expiredRef.current) {
      return;
    }
    expiredRef.current = true;
    if (onExpired !== undefined) {
      onExpired();
      return;
    }
    const back = `${window.location.pathname}${window.location.search}`;
    window.location.assign(`${loginUrl}?return_to=${encodeURIComponent(back)}`);
  }, [loginUrl, onExpired]);

  const refresh = useCallback(async () => {
    try {
      const res = await fetch(sessionUrl, {
        method: "GET",
        credentials: "same-origin",
        cache: "no-store",
        headers: { Accept: "application/json" },
      });
      if (res.status === 401) {
        expire();
        return;
      }
      if (res.ok) {
        const s = parseStatus((await res.json()) as StatusBody, Date.now());
        if (s !== null) {
          setStatus(s);
        }
      }
    } catch {
      // Jaringan putus: status lama dipakai sampai pemeriksaan berikutnya.
    }
  }, [sessionUrl, expire]);

  useEffect(() => {
    void refresh();
    const poll = window.setInterval(() => void refresh(), pollIntervalMs);
    const tick = window.setInterval(() => {
      setNow(Date.now());
    }, 1000);
    const visible = () => {
      if (document.visibilityState === "visible") {
        void refresh();
      }
    };
    document.addEventListener("visibilitychange", visible);
    return () => {
      window.clearInterval(poll);
      window.clearInterval(tick);
      document.removeEventListener("visibilitychange", visible);
    };
  }, [refresh, pollIntervalMs]);

  const serverNow = now + (status?.skewMs ?? 0);
  const idleLeft = status === null ? Infinity : status.idleExpiresAt - serverNow;
  const absoluteLeft = status === null ? Infinity : status.absoluteExpiresAt - serverNow;
  const absoluteBinds = status !== null && status.absoluteExpiresAt <= status.idleExpiresAt;
  const left = Math.min(idleLeft, absoluteLeft);
  const warningMs = status?.warningMs ?? 0;
  const mode: "none" | "idle" | "absolute" =
    status === null || left > warningMs ? "none" : absoluteBinds ? "absolute" : "idle";
  const open = mode === "idle" || (mode === "absolute" && !absoluteDismissed);

  useEffect(() => {
    if (status !== null && left <= 0) {
      // Pastikan ke server (aktivitas di tab lain dapat memperpanjang sesi).
      void refresh().then(() => {
        if (!expiredRef.current && Date.now() + status.skewMs >= Math.min(status.idleExpiresAt, status.absoluteExpiresAt)) {
          expire();
        }
      });
    }
  }, [status, left, refresh, expire]);

  useEffect(() => {
    if (open) {
      dialogRef.current?.querySelector<HTMLButtonElement>("[data-aksi-utama]")?.focus();
    }
  }, [open, mode]);

  const tetapMasuk = async () => {
    if (status === null) {
      return;
    }
    setExtending(true);
    setExtendFailed(false);
    try {
      const res = await fetch(extendUrl, {
        method: "POST",
        credentials: "same-origin",
        cache: "no-store",
        headers: { Accept: "application/json", "X-CSRF-Token": status.csrfToken },
      });
      if (res.status === 401) {
        expire();
        return;
      }
      const s = res.ok ? parseStatus((await res.json()) as StatusBody, Date.now()) : null;
      if (s === null) {
        setExtendFailed(true);
        return;
      }
      setStatus(s);
      setNow(Date.now());
    } catch {
      setExtendFailed(true);
    } finally {
      setExtending(false);
    }
  };

  if (!open || status === null) {
    return null;
  }
  return (
    <div className="rs-session-dialog__scrim" role="presentation">
      <div
        ref={dialogRef}
        className="rs-card rs-session-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="rs-session-dialog-judul"
        aria-describedby="rs-session-dialog-isi"
      >
        <h2 className="rs-session-dialog__judul" id="rs-session-dialog-judul">
          Sesi akan berakhir dalam 2 menit
        </h2>
        <p className="rs-session-dialog__isi" id="rs-session-dialog-isi">
          {mode === "idle"
            ? 'Anda tidak aktif selama hampir 30 menit. Pilih "Tetap masuk" untuk melanjutkan tanpa kehilangan isian.'
            : "Batas sesi 12 jam tercapai dan tidak dapat diperpanjang. Simpan pekerjaan Anda, lalu masuk kembali."}
        </p>
        <p className="rs-session-dialog__sisa">Sisa waktu: {formatSisa(left)}</p>
        {extendFailed ? (
          <p className="rs-session-dialog__galat" role="alert">
            Sesi gagal diperpanjang. Coba lagi.
          </p>
        ) : null}
        <form className="rs-session-dialog__aksi" method="post" action={logoutUrl}>
          <input type="hidden" name="csrf_token" value={status.csrfToken} />
          <Button variant="text" type="submit">
            Keluar sekarang
          </Button>
          {mode === "idle" ? (
            <Button
              data-aksi-utama=""
              variant="primary"
              loading={extending}
              onClick={() => void tetapMasuk()}
            >
              Tetap masuk
            </Button>
          ) : (
            <Button
              data-aksi-utama=""
              variant="primary"
              onClick={() => {
                setAbsoluteDismissed(true);
              }}
            >
              Mengerti
            </Button>
          )}
        </form>
      </div>
    </div>
  );
}
