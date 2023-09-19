import { marked } from 'https://cdn.jsdelivr.net/npm/marked@9/+esm'

const inscription = document.documentElement.dataset.inscription;
const response = await fetch(`/content/${inscription}`);
const markdown = await response.text();
document.body.innerHTML = marked.parse(markdown);
