import hljs from '/content/e26e96915acbaed1888fdeb72402e9eb21f83e533ee26fc7101e631b3f78eae6i0';
import javascript from '/content/7a115d8cb6f6f9feeca922cdeed30368db49d97ccb607ce4bcd6f94a7917555fi0';
import yaml from '/content/0430c866bc414f8e744bb23aabc668c23ab8b71071e533fb4ee6e0832fb5cc22i0';
import css from '/content/ca08f3ab2402161e6efdc2f4031793c237061fb2c9f561a0830618acdf970d5fi0';
import json from '/content/ea8e6708b2ba4fb5cc21ef21c330b6e650ca04ca01d02d3462ef7d250ec27eb4i0';

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
