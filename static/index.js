let next = document.querySelector("a.next");

if (next) {
  window.addEventListener("keydown", e => {
    if (e.key == "ArrowRight") {
      window.location = next.href;
    }
  });
}

let previous = document.querySelector("a.previous");

if (previous) {
  window.addEventListener("keydown", e => {
    if (e.key == "ArrowLeft") {
      window.location = previous.href;
    }
  });
}
