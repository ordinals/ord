function resize() {
  if (
    body.clientWidth * window.devicePixelRatio < img.naturalWidth
    || body.clientHeight * window.devicePixelRatio < img.naturalHeight
  ) {
    body.style.imageRendering = 'auto';
  } else {
    body.removeAttribute('style');
  }
}

let body = document.body;
let img = document.getElementsByTagName('img')[0];

(new ResizeObserver(resize)).observe(body);
