import { lex } from "wasm-jelly";

let inputTerminalUntyped = true;

let inputTerminal = document.getElementById("terminal-in");
let outputTerminal = document.getElementById("terminal-out");

const runBtn = document.getElementById("run-button");

runBtn.onclick = (event) => {
  outputTerminal.textContent = lex(inputTerminal.textContent);
};  



inputTerminal.onfocus = (event) => {
  if(inputTerminalUntyped) {
    inputTerminal.textContent = "";
    inputTerminalUntyped = false;
  }
}

// export NODE_OPTIONS=--openssl-legacy-provider

// write rust
// wasm-pack build
// write javascript
// npm run start (from .../www/)