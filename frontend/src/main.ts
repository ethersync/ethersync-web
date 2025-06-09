import './style.css'
import typescriptLogo from './typescript.svg'
import viteLogo from '/vite.svg'
import { Counter } from 'iroh-web-wasm';

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div>
    <a href="https://vite.dev" target="_blank">
      <img src="${viteLogo}" class="logo" alt="Vite logo" />
    </a>
    <a href="https://www.typescriptlang.org/" target="_blank">
      <img src="${typescriptLogo}" class="logo vanilla" alt="TypeScript logo" />
    </a>
    <h1>Vite + TypeScript</h1>
    <div class="card">
      <button id="counter" type="button"></button>
    </div>
    <p class="read-the-docs">
      Click on the Vite and TypeScript logos to learn more
    </p>
  </div>
`
const counter = new Counter();
const button = document.querySelector<HTMLButtonElement>('#counter')!;
button.innerText = counter.getText();

button.addEventListener("click", () => {
    counter.increase();
    button.innerText = counter.getText();
});
