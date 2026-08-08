import { invoke } from "@tauri-apps/api/core";

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
