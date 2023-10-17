import { marked } from '/content/cf7ef67632861da5a59a716ad5ee398054f455c0011a7b6aa12c5985c1b8b0b8i0'

const inscription = document.documentElement.dataset.inscription;
const response = await fetch(`/content/${inscription}`);
const markdown = await response.text();
document.body.innerHTML = marked.parse(markdown);
