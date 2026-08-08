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
  panel_type: "bt" | "1panel" | null;
  panel_url: string | null;
  panel_session_ref: string | null;
  created_at: string;
  updated_at: string;
}

export async function listHosts(): Promise<Host[]> {
  return invoke<Host[]>("list_hosts");
}

export async function addHost(
  host: Omit<Host, "id" | "created_at" | "updated_at" | "panel_session_ref" | "auth_ref">,
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
  online: boolean;
}

export async function collectMetrics(id: string): Promise<HostMetrics> {
  return invoke<HostMetrics>("collect_metrics", { id });
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
