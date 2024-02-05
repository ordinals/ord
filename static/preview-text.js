const inscription = document.documentElement.dataset.inscription;

const response = await fetch(`/content/${inscription}`);
const text = await response.text();
for (const pre of document.querySelectorAll('pre')) {
  try {
    pre.textContent = JSON.stringify(JSON.parse(text), null, 2);
  } catch (e) {
    pre.textContent = text;
  }
}

let pre = document.querySelector('body > pre');
pre.style.fontSize = '12px';
pre.style.opacity = 1;
pre.style.lineHeight = '1.2';
pre.style.whiteSpace = 'pre-wrap';
