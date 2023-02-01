for (let time of document.body.getElementsByTagName('time')) {
  time.setAttribute('title', new Date(time.textContent));
}

let next = document.querySelector('a.next');
let prev = document.querySelector('a.prev');

window.addEventListener('keydown', e => {
  if (document.activeElement.tagName == 'INPUT') {
    return;
  }

  switch (e.key) {
    case 'ArrowRight':
      if (next) {
        window.location = next.href;
      }
      return;
    case 'ArrowLeft':
      if (prev) {
        window.location = prev.href;
      }
      return;
  }
});
