const inscription = document.documentElement.dataset.inscription;
const response = await fetch(`/content/${inscription}`);
const fontFile = await response.blob();

const style = document.createElement('style');
const contentType = response.headers.get("content-type");
let fontType = contentType.split("/")[1];
switch (fontType) {
    case "ttf":
        fontType = "truetype";
        break;
    case "otf":
        fontType = "opentype";
        break;
}
style.innerHTML = `
@font-face {
    font-family: 'CustomFont';
    src: url(${URL.createObjectURL(fontFile)}) format('${fontType}');
}`;

document.head.appendChild(style);

const textBox = document.createElement('textarea');
textBox.value = 'The Quick Brown Fox Jumps Over The Lazy Dog';
document.body.appendChild(textBox);
