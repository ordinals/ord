import pdfjs from 'https://cdn.jsdelivr.net/npm/pdfjs-dist@3.2.146/+esm';

pdfjs.GlobalWorkerOptions.workerSrc = 'https://cdn.jsdelivr.net/npm/pdfjs-dist@3.2.146/build/pdf.worker.min.js';

let canvas = document.querySelector('canvas');

let pdf = await pdfjs.getDocument(`/content/${canvas.dataset.inscription}`).promise;

let page = await pdf.getPage(1);

let scale = window.devicePixelRatio || 1;

let viewport = page.getViewport({ scale });

canvas.width = Math.ceil(viewport.width * scale);

canvas.height = Math.ceil(viewport.height * scale);

page.render({
  canvasContext: canvas.getContext('2d'),
  transform: [scale, 0, 0, scale, 0, 0],
  viewport,
});
