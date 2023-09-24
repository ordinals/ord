import hljs from 'https://cdn.jsdelivr.net/npm/highlightjs@9.16.2/+esm'

const inscription = document.documentElement.dataset.inscription;
const response = await fetch(`/content/${inscription}`);
const code = await response.text();
const codeBlock = document.createElement('code');
codeBlock.innerHTML = hljs.highlightAuto(code).value;
document.body.appendChild(codeBlock);
