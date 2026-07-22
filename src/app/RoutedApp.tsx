import { lazy, Suspense } from "react";

const App = lazy(() => import("../App"));
const EmbeddedQuotaOverlayApp = lazy(() =>
  import("./EmbeddedQuotaOverlayApp").then(({ EmbeddedQuotaOverlayApp }) => ({
    default: EmbeddedQuotaOverlayApp,
  })),
);
const EmbeddedTrayPanelApp = lazy(() =>
  import("./EmbeddedTrayPanelApp").then(({ EmbeddedTrayPanelApp }) => ({
    default: EmbeddedTrayPanelApp,
  })),
);

const EMBEDDED_VIEW = new URLSearchParams(window.location.search).get("view");

if (EMBEDDED_VIEW) {
  document.documentElement.classList.add("embedded-quota-view");
}

export default function RoutedApp() {
  let view = <App />;

  if (EMBEDDED_VIEW === "quota-overlay") {
    view = <EmbeddedQuotaOverlayApp />;
  }

  if (EMBEDDED_VIEW === "tray-panel") {
    view = <EmbeddedTrayPanelApp />;
  }

  return <Suspense fallback={null}>{view}</Suspense>;
}
