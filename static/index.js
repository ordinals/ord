for (let time of document.body.getElementsByTagName('time')) {
  time.setAttribute('title', new Date(time.textContent));
}

let next = document.querySelector('a.next');
let prev = document.querySelector('a.prev');

window.addEventListener('keydown', e => {
  if (document.activeElement.tagName === 'INPUT') {
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


document.addEventListener("DOMContentLoaded", function() {
  if ('IntersectionObserver' in window) {
    let observer = new IntersectionObserver((entries, observer) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          let iframe = entry.target;
          iframe.src = iframe.dataset.src;
          observer.unobserve(iframe);
        }
      });
    }, {
      threshold: 0.1,
    });

    document.querySelectorAll('.lazyload-iframe').forEach(iframe => {
      observer.observe(iframe);
    });
  } else {
    document.querySelectorAll('.lazyload-iframe').forEach(iframe => {
      iframe.src = iframe.dataset.src;
    });
  }
});