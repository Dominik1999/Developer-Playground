import React, { useState, useEffect } from "react";
import init, { Outputs, execute } from "miden-wasm";
import { defaultNoteScript, defaultAccountCode, defaultTransactionScript, defaultBasicWallet, defaultBasicAuthentication } from "./scriptDefaults";

function App() {
  const [outputs, setOutputs] = useState(null);
  const [noteScript, setNoteScript] = useState(defaultNoteScript);
  const [noteInputs, setNoteInputs] = useState([
    "10376293541461622847", "", "", ""
  ]);
  const [assetAmount, setAssetAmount] = useState("");
  const [wallet, setWallet] = useState(true);
  const [auth, setAuth] = useState(true);
  const [accountCode, setAccountCode] = useState(defaultAccountCode);
  const [transactionScript, setTransactionScript] = useState(defaultTransactionScript);
  const [wasmLoaded, setWasmLoaded] = useState(false);
  const [error, setError] = useState(null);

  const handleNoteInputChange = (index, value) => {
    const updatedNoteInputs = [...noteInputs];
    updatedNoteInputs[index] = /^\d*$/.test(value) ? value : "0";
    setNoteInputs(updatedNoteInputs);
  };

  useEffect(() => {
    init()
      .then(() => {
        setWasmLoaded(true);
        console.log("WASM initialized successfully");
      })
      .catch((error) => {
        console.error("Failed to initialize WASM:", error);
        setError("Failed to initialize WASM: " + error.message);
      });
  }, []);

  const handleHash = async () => {
    if (!wasmLoaded) {
      setError("WASM not initialized yet");
      return;
    }
    setError(``);
    setOutputs(``);

    try {
      const noteInputsBigInt = noteInputs
        .map((input) => (input.trim() !== "" ? BigInt(input) : null))
        .filter((input) => input !== null)
        .slice(0, 4);

      const assetAmountValue = assetAmount ? BigInt(assetAmount) : null;

      console.log("Account Code:", accountCode);
      console.log("Note Script:", noteScript);
      console.log("Note Inputs:", noteInputsBigInt);
      console.log("Transaction Script:", transactionScript);
      console.log("Asset Amount:", assetAmountValue);
      console.log("Wallet Enabled:", wallet);
      console.log("Auth Enabled:", auth);

      setOutputs(null);

      const result = execute(
        accountCode,
        noteScript,
        noteInputsBigInt,
        transactionScript,
        assetAmountValue, // Inject asset amount
        wallet, // Inject wallet toggle
        auth // Inject auth toggle
      );
      setOutputs(result);
      console.log("Execution result:", result);
    } catch (error) {
      console.error("Execution failed:", error);
      setError(`Execution failed: ${error.message || error}`);
    }
  };

  return (
    <div className="App">
      <h1>Developer Playground</h1>
      <div style={{ display: "flex", gap: "20px" }}>
        <textarea
          placeholder="Type your note_script here..."
          value={noteScript}
          onChange={(e) => setNoteScript(e.target.value)}
          rows={20}
          cols={50}
        />
        <textarea
          placeholder="Type your transactionScript here..."
          value={transactionScript}
          onChange={(e) => setTransactionScript(e.target.value)}
          rows={20}
          cols={50}
        />
        <div>
          <h2>Note Inputs</h2>
          <table>
            <tbody>
              {noteInputs.map((input, index) => (
                <tr key={index}>
                  <td>
                    <input
                      type="number"
                      value={input}
                      onChange={(e) =>
                        handleNoteInputChange(index, e.target.value)
                      }
                      placeholder={`Input ${index + 1}`}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <h3>Asset Amount</h3>
          <input
            type="number"
            value={assetAmount}
            onChange={(e) => setAssetAmount(e.target.value)}
            placeholder="Enter asset amount"
          />
        </div>
      </div>
      <div style={{ marginTop: "20px" }}>
        <h3>Toggle Options</h3>
        <label>
          Wallet:
          <input
            type="checkbox"
            checked={wallet}
            onChange={(e) => setWallet(e.target.checked)}
          />
        </label>
        <label style={{ marginLeft: "20px" }}>
          Auth:
          <input
            type="checkbox"
            checked={auth}
            onChange={(e) => setAuth(e.target.checked)}
          />
        </label>
      </div>
      <div style={{ display: "flex", gap: "20px", marginTop: "20px" }}>
        <textarea
          placeholder="Type your accountCode here..."
          value={accountCode}
          onChange={(e) => setAccountCode(e.target.value)}
          rows={20}
          cols={50}
        />
        <textarea value={defaultBasicWallet} rows={20} cols={50} readOnly />
        <textarea value={defaultBasicAuthentication} rows={20} cols={50} readOnly />
      </div>
      <br />
      <button onClick={handleHash}>Execute Transaction</button>
      <button
        onClick={() => window.location.reload()}
        style={{ marginLeft: "10px" }}
      >
        Reload Page
      </button>

      {outputs && (
        <div>
          <h3>Outputs:</h3>
          <ul>
            <li>
              <strong>Account Code Commitment:</strong> {outputs.account_code_commitment}
            </li>
            <li>
              <strong>Account Delta Nonce:</strong> {outputs.account_delta_nonce}
            </li>
            <li>
              <strong>Account Delta Storage:</strong> {outputs.account_delta_storage}
            </li>
            <li>
              <strong>Account Delta Vault:</strong> {outputs.account_delta_vault}
            </li>
            <li>
              <strong>Account Hash:</strong> {outputs.account_hash}
            </li>
            <li>
              <strong>Account Storage Commitment:</strong> {outputs.account_storage_commitment}
            </li>
            <li>
              <strong>Account Vault Commitment:</strong> {outputs.account_vault_commitment}
            </li>
            <li>
              <strong>Cycle Count:</strong> {outputs.cycle_count}
            </li>
            <li>
              <strong>Trace Length:</strong> {outputs.trace_length}
            </li>
          </ul>
        </div>
      )}
      {error && (
        <div style={{ color: "red", marginTop: "20px" }}>
          <h3>Error:</h3>
          <p>{error}</p>
        </div>
      )}
    </div>
  );
}

export default App;
