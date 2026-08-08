import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-6 bg-zinc-950 text-zinc-100">
      <h1 className="text-3xl font-bold tracking-tight text-indigo-400">
        Tianxuan Desktop
      </h1>
      <p className="text-zinc-400">BT &amp; 1Panel server management tool</p>

      <form
        className="flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          className="rounded-md border border-zinc-700 bg-zinc-900 px-3 py-2 text-sm outline-none focus:border-indigo-500"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button
          type="submit"
          className="rounded-md bg-indigo-500 px-4 py-2 text-sm font-medium text-white transition hover:bg-indigo-400"
        >
          Greet
        </button>
      </form>
      <p className="text-sm text-zinc-500">{greetMsg}</p>
    </main>
  );
}

export default App;
