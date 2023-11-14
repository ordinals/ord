const inscription = document.documentElement.dataset.inscription;

const response = await fetch(`/content/${inscription}`);
const text = await response.text();
for (const pre of document.querySelectorAll('pre')) {
  pre.textContent = text;
}

let pre = document.querySelector('body > pre');
let { width, height } = pre.getBoundingClientRect();
let columns = width / 16;
let rows = height / 16;
pre.style.fontSize = `min(${95/columns}vw, ${95/rows}vh)`;
pre.style.opacity = 1;
