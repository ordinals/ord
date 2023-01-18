let next = document.querySelector('a.next');

if (next) {
  window.addEventListener('keydown', e => {
    if (e.key == 'ArrowRight') {
      window.location = next.href;
    }
  });
}

let prev = document.querySelector('a.prev');

if (prev) {
  window.addEventListener('keydown', e => {
    if (e.key == 'ArrowLeft') {
      window.location = prev.href;
    }
  });
}
