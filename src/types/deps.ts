/** Dependency check result returned from the backend. */
export interface Deps {
  rclone_installed: boolean;
  macfuse_installed: boolean;
  rclone_version: string | null;
}
