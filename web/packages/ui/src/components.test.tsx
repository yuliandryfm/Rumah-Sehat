import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { Alert, BADGE_SYMBOL, Badge, Button, Card, Checkbox, Skeleton, TextField } from "./index.ts";

afterEach(cleanup);

describe("Button", () => {
  test("default → tombol type=button dengan kelas varian primary, dapat diklik", () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Bayar</Button>);
    const tombol = screen.getByRole("button", { name: "Bayar" });
    expect(tombol).toHaveProperty("type", "button");
    expect(tombol.className).toContain("rs-btn--primary");
    fireEvent.click(tombol);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  test("varian destructive → kelas varian sesuai", () => {
    render(<Button variant="destructive">Hapus</Button>);
    expect(screen.getByRole("button", { name: "Hapus" }).className).toContain("rs-btn--destructive");
  });

  test("nonaktif → atribut disabled dan klik tidak memanggil handler", () => {
    const onClick = vi.fn();
    render(
      <Button disabled onClick={onClick}>
        Kirim
      </Button>,
    );
    const tombol = screen.getByRole("button", { name: "Kirim" });
    expect(tombol).toHaveProperty("disabled", true);
    fireEvent.click(tombol);
    expect(onClick).not.toHaveBeenCalled();
  });

  test("memuat → label tetap, aria-busy, terkunci, klik ganda tidak mengirim ulang (P5, KU-05)", () => {
    const onClick = vi.fn();
    render(
      <Button loading onClick={onClick}>
        Bayar
      </Button>,
    );
    const tombol = screen.getByRole("button", { name: "Memproses..." });
    expect(tombol.getAttribute("aria-busy")).toBe("true");
    expect(tombol.getAttribute("aria-disabled")).toBe("true");
    expect(tombol).toHaveProperty("disabled", false); // tetap dapat difokus
    fireEvent.click(tombol);
    fireEvent.click(tombol);
    expect(onClick).not.toHaveBeenCalled();
    expect(screen.queryByText("Bayar")).toBeNull();
  });

  test("memuat dengan label khusus → memakai label itu", () => {
    render(
      <Button loading loadingLabel="Mengirim...">
        Kirim
      </Button>,
    );
    expect(screen.getByRole("button", { name: "Mengirim..." })).toBeDefined();
  });

  test("tanpa loading → tidak ada aria-busy atau aria-disabled", () => {
    render(<Button>Ok</Button>);
    const tombol = screen.getByRole("button", { name: "Ok" });
    expect(tombol.hasAttribute("aria-busy")).toBe(false);
    expect(tombol.hasAttribute("aria-disabled")).toBe(false);
  });
});

describe("TextField", () => {
  test("label terhubung ke isian dan terlihat", () => {
    render(<TextField label="Nama lengkap" />);
    expect(screen.getByLabelText("Nama lengkap")).toBeDefined();
  });

  test("galat → aria-invalid, pesan teks dengan ikon, dan aria-describedby menunjuk pesan", () => {
    render(<TextField label="Nomor telepon" hint="Contoh 0812" error="Nomor harus diawali 08." />);
    const isian = screen.getByLabelText("Nomor telepon");
    expect(isian.getAttribute("aria-invalid")).toBe("true");
    const ids = (isian.getAttribute("aria-describedby") ?? "").split(" ");
    expect(ids).toHaveLength(2);
    const pesan = ids.map((id) => document.getElementById(id)?.textContent ?? "");
    expect(pesan.join("|")).toContain("Nomor harus diawali 08.");
    expect(pesan.join("|")).toContain("Contoh 0812");
  });

  test("tanpa galat → tidak ada aria-invalid dan tidak ada aria-describedby", () => {
    render(<TextField label="Kota" />);
    const isian = screen.getByLabelText("Kota");
    expect(isian.hasAttribute("aria-invalid")).toBe(false);
    expect(isian.hasAttribute("aria-describedby")).toBe(false);
  });

  test("galat berupa string kosong → tidak dianggap galat", () => {
    render(<TextField label="Kota" error="" />);
    expect(screen.getByLabelText("Kota").hasAttribute("aria-invalid")).toBe(false);
  });

  test("nonaktif → disabled; paste tidak diblokir (tidak ada handler onPaste bawaan)", () => {
    render(<TextField label="Kode" disabled />);
    expect(screen.getByLabelText("Kode")).toHaveProperty("disabled", true);
    const aktif = render(<TextField label="Password" type="password" />);
    const isian = aktif.getByLabelText("Password");
    const dibatalkan = !fireEvent.paste(isian, { clipboardData: { getData: () => "rahasia" } });
    expect(dibatalkan).toBe(false);
  });

  test("dua isian → id unik", () => {
    render(
      <>
        <TextField label="A" />
        <TextField label="B" />
      </>,
    );
    expect(screen.getByLabelText("A").id).not.toBe(screen.getByLabelText("B").id);
  });
});

describe("Checkbox", () => {
  test("default → tidak tercentang otomatis (KEPUTUSAN-177)", () => {
    render(<Checkbox label="Saya menyetujui" />);
    expect(screen.getByRole("checkbox", { name: "Saya menyetujui" })).toHaveProperty("checked", false);
  });

  test("klik pengguna → tercentang; klik lagi → tidak", () => {
    render(<Checkbox label="Setuju" />);
    const kotak = screen.getByRole("checkbox", { name: "Setuju" });
    fireEvent.click(kotak);
    expect(kotak).toHaveProperty("checked", true);
    fireEvent.click(kotak);
    expect(kotak).toHaveProperty("checked", false);
  });

  test("galat → aria-invalid dan pesan teks terhubung", () => {
    render(<Checkbox label="Setuju" error="Centang untuk melanjutkan." />);
    const kotak = screen.getByRole("checkbox", { name: "Setuju" });
    expect(kotak.getAttribute("aria-invalid")).toBe("true");
    const id = kotak.getAttribute("aria-describedby") ?? "";
    expect(document.getElementById(id)?.textContent).toContain("Centang untuk melanjutkan.");
  });

  test("nonaktif → disabled", () => {
    render(<Checkbox label="Tidak tersedia" disabled />);
    expect(screen.getByRole("checkbox", { name: "Tidak tersedia" })).toHaveProperty("disabled", true);
  });
});

describe("Badge", () => {
  test.each(Object.entries(BADGE_SYMBOL))("nada %s → simbol %s tampil bersama teks status, simbol disembunyikan dari pembaca layar", (tone, simbol) => {
    const { container } = render(<Badge tone={tone as keyof typeof BADGE_SYMBOL}>Nama Status</Badge>);
    expect(container.textContent).toBe(`${simbol}Nama Status`);
    expect(container.querySelector("[aria-hidden='true']")?.textContent).toBe(simbol);
    expect(container.querySelector(".rs-badge")?.className).toContain(`rs-badge--${tone}`);
  });

  test("lima nada punya lima simbol berbeda (warna bukan satu-satunya penanda)", () => {
    expect(new Set(Object.values(BADGE_SYMBOL)).size).toBe(5);
  });
});

describe("Alert", () => {
  test("danger → role=alert; info, success, warning → role=status (SC 4.1.3)", () => {
    const { rerender } = render(
      <Alert variant="danger" title="Gagal">
        isi
      </Alert>,
    );
    expect(screen.getByRole("alert").textContent).toContain("Gagal");
    for (const variant of ["info", "success", "warning"] as const) {
      rerender(
        <Alert variant={variant} title="Judul">
          isi
        </Alert>,
      );
      expect(screen.getByRole("status").className).toContain(`rs-alert--${variant}`);
      expect(screen.queryByRole("alert")).toBeNull();
    }
  });

  test("tanpa isi → hanya judul", () => {
    render(<Alert variant="info" title="Hanya judul" />);
    expect(screen.getByRole("status").querySelector(".rs-alert__text")).toBeNull();
  });
});

describe("Skeleton", () => {
  test("memuat → teks tersembunyi 'Memuat' satu kali dan N baris dekoratif", () => {
    const { container } = render(<Skeleton lines={3} />);
    expect(screen.getAllByText("Memuat")).toHaveLength(1);
    expect(container.querySelectorAll(".rs-skeleton__bar")).toHaveLength(3);
    expect(container.querySelectorAll(".rs-skeleton__bar[aria-hidden='true']")).toHaveLength(3);
  });

  test("lines tidak valid (0 atau negatif) → minimal satu baris", () => {
    const { container } = render(<Skeleton lines={0} />);
    expect(container.querySelectorAll(".rs-skeleton__bar")).toHaveLength(1);
  });
});

describe("Card", () => {
  test("statis → div tanpa kelas pressable", () => {
    const { container } = render(<Card>Isi</Card>);
    expect(container.firstElementChild?.tagName).toBe("DIV");
    expect(container.firstElementChild?.className).not.toContain("pressable");
  });

  test("dengan href → tautan dapat ditekan", () => {
    render(<Card href="/pesanan/1">Lihat pesanan</Card>);
    const tautan = screen.getByRole("link", { name: "Lihat pesanan" });
    expect(tautan.getAttribute("href")).toBe("/pesanan/1");
    expect(tautan.className).toContain("rs-card--pressable");
  });

  test("memuat → isi diganti kerangka dan aria-busy", () => {
    const { container } = render(<Card loading>Rahasia isi</Card>);
    expect(screen.queryByText("Rahasia isi")).toBeNull();
    expect(screen.getByText("Memuat")).toBeDefined();
    expect(container.firstElementChild?.getAttribute("aria-busy")).toBe("true");
  });
});
