import React, { useState, useEffect } from 'react';
import init, { example } from 'miden-wasm'; // Import the WASM bindings
import { defaultNoteScript, defaultAccountCode, defaultTransactionScript } from './scriptDefaults';

function App() {
  const [hashValue, setHashValue] = useState('');
  const [noteScript, setNoteScript] = useState(defaultNoteScript);
  const [accountCode, setAccountCode] = useState(defaultAccountCode);
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

  const handleHash = () => {
    if (!wasmLoaded) {
      console.error("WASM module is not loaded yet");
      return;
    }

    // Call the example function with the note_script
    console.log("Account Code:", accountCode);
    console.log("Note Script:", noteScript);
    console.log("Transaction Script:", transactionScript);
    const result = example(accountCode, noteScript, transactionScript);
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
      <br />
      <button onClick={handleHash}>Execute Transaction!!1</button>
      <p>Result: {hashValue}</p>
    </div>
  );
}

export default App;
