import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { listen } from "@tauri-apps/api/event";

interface InterpreterMetadata {
  path: string;
  version?: string;
  environment_type?: string;
}

function App() {
  const [interpreters, setInterpreters] = useState<InterpreterMetadata[] | string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [logs, setLogs] = useState<string[]>([]); // <-- Live logs state for UI

  async function fetchInterpreters() {
    setLoading(true);
    setError("");

    try {
      const res = await invoke<any[]>("test_pyre_engine");
      setInterpreters(res);
    } catch (err) {
      console.error(err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  async function runTestScript() {
    try {
      // Use absolute path of python discovered or standard one
      const pythonPath = "/usr/bin/python3";
      const scriptPath = "/home/shabd-vasudeva/pyre-ide/src-tauri/src/test_script.py";

      const pid = await invoke("run_python_script_command", {
        pythonPath,
        scriptPath,
      });
      console.log("Started Python process with PID:", pid);
    } catch (err) {
      console.error("Failed to run script:", err);
      setError(String(err));
    }
  }

  useEffect(() => {
    fetchInterpreters();

    // Listen to 'python-stdout' emitted from Rust backend
    const unlistenStdout = listen<string>("python-stdout", (event) => {
      console.log("EVENT RECEIVED STDOUT:", event.payload);
      setLogs((prev) => [...prev, `[STDOUT]: ${event.payload}`]);
    });

    const unlistenStderr = listen<string>("python-stderr", (event) => {
      console.error("EVENT RECEIVED STDERR:", event.payload);
      setLogs((prev) => [...prev, `[STDERR]: ${event.payload}`]);
    });

    return () => {
      unlistenStdout.then((f) => f());
      unlistenStderr.then((f) => f());
    };
  }, []);

  return (
    <main className="container">
      <h1>Pyre IDE - Python Engine Bridge</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <div style={{ marginTop: "20px", textAlign: "left" }}>
        <button onClick={runTestScript} style={{ marginBottom: "15px", marginRight: "10px" }}>
          Run Test Python Script
        </button>
        <button onClick={fetchInterpreters} style={{ marginBottom: "15px" }}>
          Re-scan Interpreters
        </button>

        <h3>Live Script Output:</h3>
        <div style={{ background: "#1e1e1e", color: "#00ffcc", padding: "10px", borderRadius: "5px", height: "150px", overflowY: "auto", fontFamily: "monospace" }}>
          {logs.length === 0 ? <p style={{ color: "#888" }}>No output yet. Click 'Run Test Python Script'.</p> : null}
          {logs.map((log, idx) => (
            <div key={idx}>{log}</div>
          ))}
        </div>

        <h3>Discovered Python Environments:</h3>
        {loading && <p>Scanning system for Python interpreters...</p>}
        {error && <p style={{ color: "red" }}>Error: {error}</p>}

        <ul>
          {interpreters.map((item, index) => (
            <li key={index}>
              <code>{typeof item === "string" ? item : JSON.stringify(item)}</code>
            </li>
          ))}
        </ul>
      </div>
    </main>
  );
}

export default App;