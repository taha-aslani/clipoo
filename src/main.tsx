import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { TooltipProvider } from "@/components/ui/tooltip";
import App from "./App";
import "./index.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Clipoo root element was not found");
}

createRoot(rootElement).render(
  <StrictMode>
    <TooltipProvider delayDuration={200}>
      <App />
    </TooltipProvider>
  </StrictMode>,
);
