"use client";

import { Button } from "@rs/ui";
import { useEffect, useId, useRef, useState } from "react";

export interface DialogAlasanProps {
  readonly judul: string;
  readonly penjelasan: string;
  readonly labelAksi: string;
  readonly sibuk: boolean;
  readonly onBatal: () => void;
  readonly onKirim: (alasan: string) => void;
}

/**
 * Dialog hapus dengan alasan wajib (`delete_reason`, ADR-003; wireframe C14). Alasan kosong ditolak di
 * sini dengan pesan; validasi sebenarnya (1-500 karakter) tetap di API.
 */
export function DialogAlasan({ judul, penjelasan, labelAksi, sibuk, onBatal, onKirim }: DialogAlasanProps) {
  const id = useId();
  const [alasan, setAlasan] = useState("");
  const [galat, setGalat] = useState<string | null>(null);
  const ref = useRef<HTMLTextAreaElement>(null);
  useEffect(() => ref.current?.focus(), []);
  return (
    <div
      role="alertdialog"
      aria-modal="true"
      aria-labelledby={`${id}-j`}
      aria-describedby={`${id}-p`}
      className="bo-dialog"
      onKeyDown={(e) => {
        if (e.key === "Escape") onBatal();
      }}
    >
      <h2 id={`${id}-j`}>{judul}</h2>
      <p id={`${id}-p`}>{penjelasan}</p>
      <label htmlFor={`${id}-a`} className="rs-field__label">
        Alasan (wajib)
      </label>
      <textarea
        id={`${id}-a`}
        ref={ref}
        className="rs-field__input bo-textarea"
        rows={3}
        maxLength={500}
        value={alasan}
        aria-invalid={galat === null ? undefined : true}
        aria-describedby={galat === null ? undefined : `${id}-e`}
        onChange={(e) => { setAlasan(e.target.value); }}
      />
      {galat === null ? null : (
        <p id={`${id}-e`} className="rs-field__error">
          <span aria-hidden="true" className="rs-field__error-icon">
            !
          </span>
          <span>{galat}</span>
        </p>
      )}
      <div className="bo-aksi">
        <Button variant="secondary" onClick={onBatal}>
          Batal
        </Button>
        <Button
          variant="destructive"
          loading={sibuk}
          onClick={() => {
            const a = alasan.trim();
            if (a === "") {
              setGalat("Alasan wajib diisi. Tulis alasan singkat, misalnya \"konten usang\".");
              return;
            }
            setGalat(null);
            onKirim(a);
          }}
        >
          {labelAksi}
        </Button>
      </div>
    </div>
  );
}
