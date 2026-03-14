import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App";

// Detect system dark mode — WKWebView may not propagate prefers-color-scheme
const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");
function applyTheme(dark: boolean) {
  document.documentElement.classList.toggle("dark", dark);
}
applyTheme(darkQuery.matches);
darkQuery.addEventListener("change", (e) => applyTheme(e.matches));

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
