import React, { useState, useEffect } from 'react';
import init, { Outputs, execute } from 'miden-wasm'; // Import the WASM bindings
import { defaultNoteScript, defaultAccountCode, defaultTransactionScript, defaultBasicWallet, defaultBasicAuthentication } from './scriptDefaults';



function App() {
  const [outputs, setOutputs] = useState(null);
  const [noteScript, setNoteScript] = useState(defaultNoteScript);
  const [noteInputs, setNoteInputs] = useState([
    "10376293541461622847", "", "", ""
  ]);
  const [accountCode, setAccountCode] = useState(defaultAccountCode);
  const [transactionScript, setTransactionScript] = useState(defaultTransactionScript);
  const [wasmLoaded, setWasmLoaded] = useState(false);
  const [error, setError] = useState(null);  // Add error state

  // Initialize the WASM module once on component mount
  useEffect(() => {
    const initializeWasm = async () => {
      try {
        await init(); // Initialize WebAssembly
        setWasmLoaded(true); // Set to true when wasm is successfully loaded
      } catch (error) {
        console.error("Failed to initialize WASM module:", error);
      }
    };

    initializeWasm();
  }, []);

  const handleNoteInputChange = (index, value) => {
    const updatedNoteInputs = [...noteInputs];
    updatedNoteInputs[index] = /^\d*$/.test(value) ? value : "0";
    setNoteInputs(updatedNoteInputs);
  };

  const handleHash = async () => {
    if (!wasmLoaded) {
      console.error("WASM module is not loaded yet");
      return;
    }

    try {
      // Unload and reinitialize the WASM module
      console.log("Reinitializing WASM module...");
      await init(); // Reinitialize WASM module before every execution

      // Convert noteInputs to BigInt values
      const noteInputsBigInt = noteInputs
        .map(input => input.trim() !== "" ? BigInt(input) : null)
        .filter(input => input !== null)
        .slice(0, 4);

      console.log("Account Code:", accountCode);
      console.log("Note Script:", noteScript);
      console.log("Note Inputs:", noteInputsBigInt);
      console.log("Transaction Script:", transactionScript);

      // Reset previous outputs
      setOutputs(null);

      // Call the execute function asynchronously and set outputs
      const result = await execute(accountCode, noteScript, noteInputsBigInt, transactionScript);
      setOutputs(result);
    } catch (error) {
      console.error("Execution failed:", error);
      setError(`Execution failed: ${error.message || error}`);  // Set the error message
    }
  };

  return (
    <div className="App">
      <h1>Developer Playground</h1>
      <div style={{ display: 'flex', gap: '20px' }}>
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
                      onChange={(e) => handleNoteInputChange(index, e.target.value)}
                      placeholder={`Input ${index + 1}`}
                    />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
      <div style={{ display: 'flex', gap: '20px', marginTop: '20px' }}>
        <textarea
          placeholder="Type your accountCode here..."
          value={accountCode}
          onChange={(e) => setAccountCode(e.target.value)}
          rows={20}
          cols={50}
        />
        <textarea
          value={defaultBasicWallet}
          rows={20}
          cols={50}
          readOnly
        />
        <textarea
          value={defaultBasicAuthentication}
          rows={20}
          cols={50}
          readOnly
        />
      </div>
      <br />
      <button onClick={handleHash}>Execute Transaction</button>

      {outputs && (
        <div>
          <h3>Outputs:</h3>
          <p><strong>Account Delta Storage:</strong> {outputs.account_delta_storage}</p>
          <p><strong>Account Delta Vault:</strong> {outputs.account_delta_vault}</p>
          <p><strong>Account Delta Nonce:</strong> {outputs.account_delta_nonce}</p>
          <p><strong>Account Code Commitment:</strong> {outputs.account_code_commitment}</p>
          <p><strong>Account Storage Commitment:</strong> {outputs.account_storage_commitment}</p>
          <p><strong>Account Vault Commitment:</strong> {outputs.account_vault_commitment}</p>
          <p><strong>Account Hash:</strong> {outputs.account_hash}</p>
          <p><strong>Cycle Count:</strong> {outputs.cycle_count}</p>
          <p><strong>Trace Length:</strong> {outputs.trace_length}</p>
        </div>
      )}
      {/* Display the error message if there is an error */}
      {error && (
        <div style={{ color: 'red', marginTop: '20px' }}>
          <h3>Error:</h3>
          <p>{error}</p>
        </div>
      )}
    </div>
  );
}

export default App;
