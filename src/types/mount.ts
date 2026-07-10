/** Represents a single mount item (from rclone.conf). */
export interface MountItem {
  id: string;
  name: string;
  remote_path: string;
  mount_point: string;
  source: string;
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
