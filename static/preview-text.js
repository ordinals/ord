let pre = document.querySelector('body > pre');
let { width, height } = pre.getBoundingClientRect();
let columns = width / 16;
let rows = height / 16;
pre.style.fontSize = `min(${95/columns}vw, ${95/rows}vh)`;
pre.style.opacity = 1;

function decodeHtml(html) {
    var txt = document.createElement("textarea");
    txt.innerHTML = html;
    return txt.value;
}
function parseIpfsUrl(url) {
    if(!url) return;
    if(url.startsWith('ipfs://')) return url.replace('ipfs://', 'https://ipfs.io/ipfs/');
    return url;
}

async function getMetaData(url) {
 try {
    return await ((await fetch(url)).json());
 }catch(e) {
    return null;
 }
}


let result = document.getElementById('text').innerText;
try{
    const resultJson = JSON.parse(decodeHtml(result));
    result = JSON.stringify(resultJson, null, 2);
    if(resultJson.uri.endsWith('json')) {
        getMetaData(parseIpfsUrl(resultJson.uri)).then(metadata=> {
            if(metadata) {
                result = `<img src="${parseIpfsUrl(metadata.image)}" />`;
                document.getElementById('preview').innerHTML = result;
            }
        })
    }
  
  console.log('RESULT', result);
} catch(e) {
  
console.log('ERROR', e, decodeHtml(result));
}
document.getElementById('text').innerText='';
document.getElementById('preview').innerText = result;

console.log('Parseing...')
