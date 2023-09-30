import hljs from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/highlight.min.js';
import javascript from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/javascript.min.js';
import yaml from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/yaml.min.js';
import css from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/css.min.js';
import json from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/json.min.js';

hljs.registerLanguage('javascript', javascript);
hljs.registerLanguage('yaml', yaml);
hljs.registerLanguage('css', css);
hljs.registerLanguage('json', json);

const inscription = document.documentElement.dataset.inscription;
const response = await fetch(`/content/${inscription}`);
const contentType = response.headers.get("content-type");
const language = contentType.split("/")[1];
const code = await response.text();

document.body.innerHTML = `<pre><code>${hljs.highlight(code, {language, ignoreIllegals: true}).value}</code></pre>`;
