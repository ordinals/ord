let pre = document.getElementsByTagName('pre')[0];

// TODO:
// - add margins
// - make it work with noscript
// - could use a fixed font with fixed dimensions, and make it pure css
// - fix overflow cases

let rect = pre.getBoundingClientRect();

console.log('pre', rect);
console.log('body', document.body.clientWidth, document.body.clientHeight);

let last = null;

if (false) {
  pre.style.opacity = 1;
} else {
  function resize() {
    let size = Math.min(document.body.clientWidth / rect.width, document.body.clientHeight / rect.height);
    if(size != last) {
      pre.style.fontSize = `${size}em`;
      pre.style.opacity = 1;
      last = size;
    }
  }

  addEventListener("resize", resize);
  resize();
}
