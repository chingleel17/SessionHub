import App from "../App";
import { EmbeddedQuotaOverlayApp } from "./EmbeddedQuotaOverlayApp";
import { EmbeddedTrayPanelApp } from "./EmbeddedTrayPanelApp";

const EMBEDDED_VIEW = new URLSearchParams(window.location.search).get("view");

if (EMBEDDED_VIEW) {
  document.documentElement.classList.add("embedded-quota-view");
}

export default function RoutedApp() {
  if (EMBEDDED_VIEW === "quota-overlay") {
    return <EmbeddedQuotaOverlayApp />;
  }

  if (EMBEDDED_VIEW === "tray-panel") {
    return <EmbeddedTrayPanelApp />;
  }

  return <App />;
}
