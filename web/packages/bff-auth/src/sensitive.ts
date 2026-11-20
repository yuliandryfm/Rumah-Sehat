// Pembungkus nilai rahasia (padanan `foundation::Sensitive` di Rust, coding-standards §6).
// Nilai hanya dapat dibaca lewat `expose()`; konversi ke teks, JSON, dan inspeksi Node menghasilkan
// "[REDACTED]", sehingga rahasia tidak ikut tercetak di log atau pesan galat secara tidak sengaja.

const REDACTED = "[REDACTED]";

export class Sensitive<T> {
  readonly #value: T;

  constructor(value: T) {
    this.#value = value;
  }

  expose(): T {
    return this.#value;
  }

  toString(): string {
    return REDACTED;
  }

  toJSON(): string {
    return REDACTED;
  }

  [Symbol.for("nodejs.util.inspect.custom")](): string {
    return REDACTED;
  }
}
