let text = document.getElementsByTagName('pre')[0].textContent;

let canvas = document.createElement('canvas');
document.body.appendChild(canvas);

var ctx = canvas.getContext("2d");

ctx.fillText(text, 0, 200);
