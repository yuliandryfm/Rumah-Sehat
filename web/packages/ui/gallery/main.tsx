import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "../src/tokens.css";
import "../src/ui.css";
import "./galeri.css";
import { Galeri } from "./Galeri.tsx";

const akar = document.getElementById("akar");
if (akar === null) {
  throw new Error("elemen #akar tidak ada di index.html");
}
createRoot(akar).render(
  <StrictMode>
    <Galeri />
  </StrictMode>,
);
