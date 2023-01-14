let pre = document.querySelector('body > pre');
let { width, height } = pre.getBoundingClientRect();
let columns = width / 16;
let rows = height / 16;
pre.style.fontSize = `min(${100/columns}vw, ${100/rows}vh)`;
pre.style.opacity = 1;
