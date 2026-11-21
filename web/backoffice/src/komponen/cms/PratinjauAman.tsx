import { dokumenPratinjau, SANDBOX_PRATINJAU } from "../../lib/cms/pratinjau.ts";

/**
 * Pratinjau isi konten di `<iframe sandbox>` tanpa `allow-same-origin` (dan tanpa `allow-scripts`):
 * dokumen pratinjau berasal dari origin buram sehingga tidak dapat membaca cookie atau DOM backoffice
 * (US-1802 AC8, ADR-027 butir 5). Dokumen dibangun di `lib/cms/pratinjau.ts` dan dimuat lewat `srcdoc`.
 */
export function PratinjauAman({ judul, markdown }: { judul: string; markdown: string }) {
  return (
    <iframe
      title="Pratinjau konten"
      className="bo-pratinjau"
      sandbox={SANDBOX_PRATINJAU}
      referrerPolicy="no-referrer"
      srcDoc={dokumenPratinjau(judul, markdown)}
    />
  );
}
