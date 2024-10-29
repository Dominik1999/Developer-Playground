import React, { useState, useEffect } from 'react';
import init, { example } from 'miden-wasm'; // Import the WASM bindings
import { defaultNoteScript, defaultAccountCode, defaultTransactionScript } from './scriptDefaults';

function App() {
  const [hashValue, setHashValue] = useState('');
  const [noteScript, setNoteScript] = useState(defaultNoteScript);
  const [noteInputs, setNoteInputs] = useState([
    "10376293541461622847", "", "", ""
  ]);  const [accountCode, setAccountCode] = useState(defaultAccountCode);
  const [transactionScript, setTransactionScript] = useState(defaultTransactionScript);
  const [wasmLoaded, setWasmLoaded] = useState(false);

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
    // Set value only if it is a valid numeric string, otherwise default to "0"
    updatedNoteInputs[index] = /^\d*$/.test(value) ? value : "0";
    setNoteInputs(updatedNoteInputs);
  };

  const handleHash = () => {
  if (!wasmLoaded) {
    console.error("WASM module is not loaded yet");
    return;
  }

  // Filter and convert noteInputs to BigInt
  const noteInputsBigInt = noteInputs
    .map(input => input.trim() !== "" ? BigInt(input) : null) // Convert to BigInt or null if empty
    .filter(input => input !== null) // Remove nulls
    .slice(0, 4); // Limit to a maximum of 4 elements

  // Log noteInputsBigInt for debugging
  console.log("Filtered Note Inputs (BigInt):", noteInputsBigInt);
  // Call the example function with the note_script
  console.log("Account Code:", accountCode);
  console.log("Note Script:", noteScript);
  console.log("Transaction Script:", transactionScript);
  const result = example(accountCode, noteScript, noteInputsBigInt, transactionScript);
  setHashValue(result);
};

  return (
    <div className="App">
      <h1>Developer Playground</h1>
      <textarea
        placeholder="Type your note_script here..."
        value={noteScript}
        onChange={(e) => setNoteScript(e.target.value)} // Update state on change
        rows={30}
        cols={50}
      />
      <textarea
        placeholder="Type your transactionScript here..."
        value={transactionScript}
        onChange={(e) => setTransactionScript(e.target.value)} // Update state on change
        rows={30}
        cols={50}
      />
      <textarea
        placeholder="Type your accountCode here..."
        value={accountCode}
        onChange={(e) => setAccountCode(e.target.value)} // Update state on change
        rows={30}
        cols={50}
      />
      {/* Note Inputs Table */}
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
      <br />
      <button onClick={handleHash}>Execute Transaction!!1</button>
      <p>Result: {hashValue}</p>
    </div>
  );
}

export default App;
