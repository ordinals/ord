import marked from 'https://cdn.jsdelivr.net/npm/marked/marked.min.js';

async function renderMarkdown() {
  const contentDiv = document.getElementById('content');
  
  const inscriptionId = contentDiv.dataset.inscription;
  const response = await fetch(`/content/${inscriptionId}`);
  const markdownContent = await response.text();
  
  const htmlContent = marked.parse(markdownContent);
  
  contentDiv.innerHTML = htmlContent;
}

window.addEventListener('DOMContentLoaded', renderMarkdown);
