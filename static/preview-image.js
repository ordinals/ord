function isThumbnailRoute() {
  return window.location.pathname.includes('/thumbnail');
}

function resize() {
  if (
    body.clientWidth * window.devicePixelRatio < img.naturalWidth
    || body.clientHeight * window.devicePixelRatio < img.naturalHeight
  ) {
    body.style.imageRendering = 'auto';
    
    // If we're in the thumbnail route, use 'cover' to fill the entire space
    if (isThumbnailRoute()) {
      body.style.backgroundSize = 'cover';
    }
  } else {
    if (isThumbnailRoute()) {
      // Only set backgroundSize when in thumbnail route
      body.style.backgroundSize = 'cover';
    } else {
      body.removeAttribute('style');
    }
  }
}

const body = document.body;
const img = document.getElementsByTagName('img')[0];

(new ResizeObserver(resize)).observe(body);
