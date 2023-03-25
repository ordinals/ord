let pre = document.querySelector('body > pre');
let { width, height } = pre.getBoundingClientRect();
let columns = width / 16;
let rows = height / 16;
pre.style.fontSize = `min(${95/columns}vw, ${95/rows}vh)`;
pre.style.opacity = 1;

function decodeHtml(html) {
    var txt = document.createElement("textarea");
    txt.innerHTML = html;
    return txt.value;
}

let result = document.getElementById('text').innerText;
try{
  result = JSON.stringify(JSON.parse(decodeHtml(result)));
} catch(e) {
console.log('ERROR', e, decodeHtml(result));
}
document.getElementById('preview').innerText = result;

console.log('Parseing...')
