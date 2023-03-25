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

function toLink(url, label) {
    return url;
    if(!label) label = url;
    return `<a href="${url}" target="_blank">${url}</a>`;
}


let result = document.getElementById('text').innerText;
try{
    const resultJson = JSON.parse(decodeHtml(result));
    result = JSON.stringify(resultJson, null, 2);
    if(resultJson.uri.endsWith('json')) {
        getMetaData(parseIpfsUrl(resultJson.uri)).then(metadata=> {
            if(metadata) {
                result = `<div class="center">
                <img height="400" src="${parseIpfsUrl(metadata.image)}" />
                <div style="text-align:left; margin-top: 20px">`;
                if(resultJson.chain_data) {
                    result +=`Chain: ${resultJson.chain_data.chain}</br>`
                    result +=`Contract Address: ${resultJson.chain_data.contract}</br>`
                    result +=`TokenId: ${resultJson.chain_data.tokenId}</br>`
                }
                result +=`
                <span style="white-space: nowrap"><b>Info Url:</b> ${toLink(resultJson.info ?? resultJson.info_uri)}</span></br>
                <span style="white-space: nowrap"><b>Collection Url:</b> ${toLink(resultJson.collection ?? resultJson.collection_uri)}</span></br>
                </div>
                </div>`;
                document.getElementById('image').innerHTML = result;
                document.getElementById('text').innerText = '';
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
