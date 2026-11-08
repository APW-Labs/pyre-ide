import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface InterpreterMetadata {
  path: string;
  version?: string;
  environment_type?: string;
}

function App() {
  const [interpreters, setInterpreters] = useState<InterpreterMetadata[] | string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function fetchInterpreters() {
    setLoading(true);
    setError("");

    const result = await invoke("create_venv_command", {
      projectPath: "/home/shabd-vasudeva/Projects/",
      venvName: ".venv"
    });

    console.log("Venv created successfully:", result);

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

  useEffect(() => {
    fetchInterpreters();
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

        <button onClick={fetchInterpreters} style={{ marginTop: "10px" }}>
          Re-scan Interpreters
        </button>
      </div>
    </main>
  );
}

export default App;