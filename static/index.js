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

  let collapse = [];

  const PATTERNS = [
    // hash
    '[a-f0-9]{64}',
    // outpoint
    '[a-f0-9]{64}:[0-9]+',
    // satpoint
    '[a-f0-9]{64}:[0-9]+:[0-9]+',
    // ethereum address
    '0x[A-Fa-f0-9]{40}',
    // inscription id
    '[a-f0-9]{64}i[0-9]+',
    // p2pkh address
    '1[a-km-zA-HJ-NP-Z1-9]{25,33}',
    // p2wpkh address
    '(bc|bcrt|tb)1q[02-9ac-hj-np-z]{38}',
    // p2wsh address
    '(bc|bcrt|tb)1q[02-9ac-hj-np-z]{46}',
    // p2tr address
    '(bc|bcrt|tb)1p[02-9ac-hj-np-z]{58}',
    // git hash
    '[a-f0-9]{40}'
  ]

  let RE = new RegExp('^(' + PATTERNS.map((p) => '(' + p + ')').join('|') + ')$');

  document.querySelectorAll('.monospace').forEach((node) => {
    if (node.children.length > 0 || node.parentNode.tagName === 'H1') {
      return;
    }

    let text = node.textContent.trim();

    if (!RE.test(text)) {
      return;
    }

    node.dataset.original = text;
    collapse.push(node);
  });

  let context = document.createElement('canvas').getContext('2d');

  function resize() {
    for (let node of collapse) {
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
