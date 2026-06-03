/** Standard API response envelope returned by all Tauri commands. */
export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}
