import { useState } from "react";
import { lab_backend } from "declarations_lab/lab_backend";

function App() {
  const [greeting, setGreeting] = useState("");
  const [experimentResult, setExperimentResult] = useState("");
  const [comparison, setComparison] = useState("");

  // SIMPLE APPROACH - Direct returns, minimal error handling
  function handleSubmitSimple(event) {
    event.preventDefault();
    const name = event.target.elements.name.value;
    lab_backend
      .greet_simple(name)
      .then((greeting) => {
        setGreeting(`📝 Simple: ${greeting}`);
      })
      .catch((error) => {
        setGreeting(`Error: ${error}`);
      });
    return false;
  }

  function runExperimentSimple() {
    lab_backend
      .run_experiment_simple("test_experiment")
      .then((result) => {
        setExperimentResult(`📝 Simple: ${JSON.stringify(result, null, 2)}`);
      })
      .catch((error) => {
        setExperimentResult(`Error: ${error}`);
      });
  }

  // ROBUST APPROACH - Explicit error handling
  function handleSubmitRobust(event) {
    event.preventDefault();
    const name = event.target.elements.name.value;
    lab_backend
      .greet_robust(name)
      .then((greeting) => {
        setGreeting(`🛡️ Robust: ${greeting}`);
      })
      .catch((error) => {
        setGreeting(`🛡️ Robust Error: ${error}`);
      });
    return false;
  }

  function runExperimentRobust() {
    lab_backend
      .run_experiment_robust("test_experiment")
      .then((result) => {
        setExperimentResult(`🛡️ Robust: ${JSON.stringify(result, null, 2)}`);
      })
      .catch((error) => {
        setExperimentResult(`🛡️ Robust Error: ${error}`);
      });
  }

  function testErrorCases() {
    // Test validation errors with robust approach
    lab_backend
      .greet_robust("")
      .then(setGreeting)
      .catch((error) => {
        setGreeting(`🛡️ Empty name error: ${error}`);
      });

    lab_backend
      .run_experiment_robust("fail")
      .then((result) => {
        setExperimentResult(`🛡️ Success: ${JSON.stringify(result, null, 2)}`);
      })
      .catch((error) => {
        setExperimentResult(`🛡️ Simulated failure: ${error}`);
      });
  }

  function showComparison() {
    lab_backend.compare_approaches().then(setComparison);
  }

  return (
    <main>
      <h1>🧪 Lab Environment - Error Handling Comparison</h1>
      <img src="/logo2.svg" alt="DFINITY logo" />

      <div style={{ display: "flex", gap: "20px", margin: "20px 0" }}>
        <div style={{ flex: 1, border: "1px solid #ccc", padding: "10px" }}>
          <h3>📝 SIMPLE APPROACH</h3>
          <p style={{ fontSize: "0.9em", color: "#666" }}>
            Direct returns, minimal error handling. Good for prototyping and internal functions.
          </p>
          <form action="#" onSubmit={handleSubmitSimple}>
            <label htmlFor="name">Enter your name: &nbsp;</label>
            <input id="name" alt="Name" type="text" />
            <button type="submit">Test Greeting (Simple)</button>
          </form>
          <button onClick={runExperimentSimple} style={{ marginTop: "10px" }}>
            Run Experiment (Simple)
          </button>
        </div>

        <div style={{ flex: 1, border: "1px solid #ccc", padding: "10px" }}>
          <h3>🛡️ ROBUST APPROACH</h3>
          <p style={{ fontSize: "0.9em", color: "#666" }}>
            Simple error handling with Result&lt;T, String&gt;. Good for user-facing APIs and input validation.
          </p>
          <form action="#" onSubmit={handleSubmitRobust}>
            <label htmlFor="name2">Enter your name: &nbsp;</label>
            <input id="name2" alt="Name" type="text" />
            <button type="submit">Test Greeting (Robust)</button>
          </form>
          <button onClick={runExperimentRobust} style={{ marginTop: "10px" }}>
            Run Experiment (Robust)
          </button>
        </div>
      </div>

      <div style={{ margin: "20px 0" }}>
        <button onClick={testErrorCases} style={{ marginRight: "10px" }}>
          Test Error Cases
        </button>
        <button onClick={showComparison}>Show Comparison Guide</button>
      </div>

      <section id="greeting" style={{ margin: "10px 0" }}>
        <strong>Greeting Result:</strong> {greeting}
      </section>

      <section id="experiment" style={{ margin: "10px 0" }}>
        <strong>Experiment Result:</strong>
        <pre>{experimentResult}</pre>
      </section>

      {comparison && (
        <section style={{ margin: "20px 0", padding: "10px", backgroundColor: "#f0f0f0" }}>
          <strong>Comparison Guide:</strong>
          <pre>{comparison}</pre>
        </section>
      )}
    </main>
  );
}

export default App;
