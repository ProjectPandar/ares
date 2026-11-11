import { useApp } from "../state/store";

/** NotificationManager::bbl_show_3mf_warn_notification: the loaded project's invalid values until closed. */
export function LoadWarning() {
  const warning = useApp((state) => state.loadWarning);
  if (!warning) return null;
  return (
    <div className="notification warning" role="alert" data-testid="load-warning">
      <p>{warning}</p>
      <button className="notification-close" aria-label="Close" onClick={() => useApp.setState({ loadWarning: undefined })}>
        {"✕"}
      </button>
    </div>
  );
}
