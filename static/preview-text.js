let pre = document.querySelector('body > pre');
let { width, height } = pre.getBoundingClientRect();
let columns = width / 16;
let rows = height / 16;
pre.style.fontSize = `min(${95/columns}vw, ${95/rows}vh)`;
pre.style.opacity = 1;

let result = document.getElementById('text').innerText;
try{
  result = JSON.stringify(JSON.parse(result));
} catch(e) {
console.log('ERROR', e);
}
document.getElementById('preview').innerText = result;

console.log('Parseing...')
