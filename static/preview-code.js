import hljs from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/highlight.min.js';

const inscription = document.documentElement.dataset.inscription;
const language = document.documentElement.dataset.language;

const definition = await import(`https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/${language}.min.js`);

hljs.registerLanguage(language, definition.default);

const response = await fetch(`/content/${inscription}`);
const text = await response.text();
const code = document.querySelector('code');

code.innerHTML = hljs.highlight(text, {language, ignoreIllegals: true}).value;
