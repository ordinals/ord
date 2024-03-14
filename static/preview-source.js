
const definition = await import(`https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/xml.min.js`);

hljs.registerLanguage('xml', definition.default);

const inscription = document.documentElement.dataset.inscription;

const escapeHtml = (htmlCode) => {
    return htmlCode
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}


const response = await fetch(`/content/${inscription}`);
const text = await response.text();
const code = document.querySelector('code');

code.innerHTML = escapeHtml(text)