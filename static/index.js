addEventListener("DOMContentLoaded", () => {
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

  const search = document.querySelector('form[action="/search"]');
  const query = search.querySelector('input[name="query"]');

  search.addEventListener('submit', (e) => {
    if (!query.value) {
      e.preventDefault();
    }
  });

  let collapse = document.getElementsByClassName('collapse');

  let context = document.createElement('canvas').getContext('2d');

  function resize() {
    for (let node of collapse) {
      if (!('original' in node.dataset)) {
        node.dataset.original = node.textContent.trim();
      }
      let original = node.dataset.original;
      let length = original.length;
      let width = node.clientWidth;
      if (width == 0) {
        width = node.parentNode.getBoundingClientRect().width;
      }
      context.font = window.getComputedStyle(node).font;
      let capacity = width / (context.measureText(original).width / length);
      if (capacity >= length) {
        node.textContent = original
      } else {
        let count = Math.floor((capacity - 1) / 2);
        let start = original.substring(0, count);
        let end = original.substring(length - count);
        node.textContent = `${start}…${end}`;
      }
    }
  }

  function copy(e) {
    if ('original' in e.target.dataset && window.getSelection().toString().includes('…')) {
      e.clipboardData.setData('text/plain', e.target.dataset.original);
      e.preventDefault();
    }
  }

  addEventListener('resize', resize);

  addEventListener('copy', copy);

  resize();
});
