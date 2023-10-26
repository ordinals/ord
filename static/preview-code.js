import hljs from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/highlight.min.js';
import css from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/css.min.js';
import javascript from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/javascript.min.js';
import json from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/json.min.js';
import python from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/python.min.js';
import yaml from 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11/build/es/languages/yaml.min.js';

hljs.registerLanguage('css', css);
hljs.registerLanguage('javascript', javascript);
hljs.registerLanguage('json', json);
hljs.registerLanguage('python', python);
hljs.registerLanguage('yaml', yaml);

const inscription = document.documentElement.dataset.inscription;
const response = await fetch(`/content/${inscription}`);
const contentType = response.headers.get("content-type");
let language = contentType.split("/")[1];
if (language === "x-python") {
    language = "python";
}
const code = await response.text();
document.body.innerHTML = `<pre><code>${hljs.highlight(code, {language, ignoreIllegals: true}).value}</code></pre>`;
