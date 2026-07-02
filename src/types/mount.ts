/** Represents a single mount item (from rclone.conf or custom). */
export interface MountItem {
  id: string;
  name: string;
  remote_path: string;
  mount_point: string;
  source: 'config' | 'custom';
  mounted: boolean;
  config_type: string;
  extra_args: string[];
  host: string;
  user: string;
  pass: string;
  port: string;
}

/** A remote section parsed from rclone.conf. */
export interface RemoteConfig {
  name: string;
  config_type: string;
  options: Record<string, string>;
}

/** Persisted config overrides for a config-sourced remote.
 *  Note: `pass` is intentionally excluded — credentials are not
 *  persisted in localStorage for security. */
export interface SavedRemoteConfig {
  name: string;
  remote_path: string;
  mount_point: string;
  host: string;
  user: string;
  port: string;
  pass?: string;
}
