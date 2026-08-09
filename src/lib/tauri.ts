import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Host {
  id: string;
  name: string;
  address: string;
  port: number;
  username: string;
  auth_type: "key" | "password";
  auth_ref: string;
  group_name: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export async function listHosts(): Promise<Host[]> {
  return invoke<Host[]>("list_hosts");
}

export async function addHost(
  host: Omit<Host, "id" | "created_at" | "updated_at" | "auth_ref">,
  password?: string,
): Promise<Host> {
  return invoke<Host>("add_host", { host, password });
}

export async function updateHost(
  host: Host,
  password?: string,
): Promise<Host> {
  return invoke<Host>("update_host", { host, password });
}

export async function deleteHost(id: string): Promise<void> {
  return invoke("delete_host", { id });
}

export async function getHost(id: string): Promise<Host | null> {
  return invoke<Host | null>("get_host", { id });
}

export async function testConnection(id: string): Promise<void> {
  return invoke("test_connection", { id });
}

export interface ExecResult {
  stdout: string;
  stderr: string;
  exit_code: number | null;
}

export async function execOnHost(id: string, command: string): Promise<ExecResult> {
  return invoke<ExecResult>("exec_on_host", { id, command });
}

export interface HostMetrics {
  cpu_percent: number;
  mem_total_mb: number;
  mem_used_mb: number;
  mem_percent: number;
  disk_total_gb: number;
  disk_used_gb: number;
  disk_percent: number;
  load_1: number;
  load_5: number;
  load_15: number;
  io_read_kbps: number;
  io_write_kbps: number;
  net_rx_kbps: number;
  net_tx_kbps: number;
  online: boolean;
}

export async function collectMetrics(id: string): Promise<HostMetrics> {
  return invoke<HostMetrics>("collect_metrics", { id });
}

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  permissions: number;
  modified: string;
}

export async function sftpList(id: string, path: string): Promise<FileEntry[]> {
  return invoke<FileEntry[]>("sftp_list", { id, path });
}

export async function sftpUpload(
  id: string,
  local: string,
  remote: string,
): Promise<void> {
  return invoke("sftp_upload", { id, local, remote });
}

export async function sftpDownload(
  id: string,
  remote: string,
  local: string,
): Promise<void> {
  return invoke("sftp_download", { id, remote, local });
}

export async function sftpDelete(id: string, path: string): Promise<void> {
  return invoke("sftp_delete", { id, path });
}

export async function sftpRename(
  id: string,
  oldPath: string,
  newPath: string,
): Promise<void> {
  return invoke("sftp_rename", { id, oldPath, newPath });
}

export async function sftpReadText(id: string, path: string): Promise<string> {
  return invoke<string>("sftp_read_text", { id, path });
}

export async function sftpWriteText(
  id: string,
  path: string,
  content: string,
): Promise<void> {
  return invoke("sftp_write_text", { id, path, content });
}

export interface BatchResult {
  host_id: string;
  host_name: string;
  stdout: string;
  stderr: string;
  exit_code: number | null;
  success: boolean;
  elapsed_ms: number;
  error: string | null;
}

export interface CommandHistory {
  id: string;
  command: string;
  host_count: number;
  executed_at: string;
  success_count: number;
  fail_count: number;
}

export async function batchExec(
  hostIds: string[],
  cmd: string,
): Promise<BatchResult[]> {
  return invoke<BatchResult[]>("batch_exec", { hostIds, cmd });
}

export async function listCommandHistory(): Promise<CommandHistory[]> {
  return invoke<CommandHistory[]>("list_command_history");
}

export interface Panel {
  id: string;
  name: string;
  url: string;
  panel_type: "bt" | "1panel";
  session_ref: string | null;
  created_at: string;
  updated_at: string;
}

export async function listPanels(): Promise<Panel[]> {
  return invoke<Panel[]>("list_panels");
}

export async function addPanel(
  panel: Omit<Panel, "id" | "session_ref" | "created_at" | "updated_at">,
): Promise<Panel> {
  return invoke<Panel>("add_panel", { panel });
}

export async function updatePanel(panel: Panel): Promise<Panel> {
  return invoke<Panel>("update_panel", { panel });
}

export async function deletePanel(id: string): Promise<void> {
  return invoke("delete_panel", { id });
}

export async function openPanelWindow(id: string): Promise<string> {
  return invoke<string>("open_panel_window", { id });
}

export async function sshOpenSession(id: string, sessionId: string): Promise<void> {
  return invoke("ssh_open_session", { id, sessionId });
}

export async function sshWrite(sessionId: string, data: Uint8Array): Promise<void> {
  return invoke("ssh_write", { sessionId, data: Array.from(data) });
}

export async function sshResize(sessionId: string, cols: number, rows: number): Promise<void> {
  return invoke("ssh_resize", { sessionId, cols, rows });
}

export async function sshCloseSession(sessionId: string): Promise<void> {
  return invoke("ssh_close_session", { sessionId });
}

export interface TerminalOutputPayload {
  session_id: string;
  data: string;
}

export async function listenTerminalOutput(
  handler: (p: TerminalOutputPayload) => void,
): Promise<() => void> {
  return listen<TerminalOutputPayload>("terminal-output", (event) => {
    handler(event.payload);
  });
}
