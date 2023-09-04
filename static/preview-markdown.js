import { marked } from 'https://cdn.jsdelivr.net/npm/marked@8.0.0/+esm'
import DOMPurify from 'https://cdn.jsdelivr.net/npm/dompurify@3.0.5/+esm'

async function renderMarkdown() {
  const contentDiv = document.getElementById('content');
  
  const inscriptionId = contentDiv.dataset.inscription;
  const response = await fetch(`/content/${inscriptionId}`);
  const rawMarkdown = await response.text();
  
  const html = DOMPurify.sanitize(marked.parse(rawMarkdown));

  contentDiv.innerHTML = html;
}

window.addEventListener('DOMContentLoaded', renderMarkdown);
