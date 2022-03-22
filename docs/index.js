function list(outpoint) {
  return fetch(`http://api.ordinals.com:8000/list/${outpoint}`)
    .then((res) => {
      document.getElementById('result').innerHTML = res
        .json()
        .map((range) => `[${range[0]}, ${range[1]})<br>`)
        .join('');
    })
    .catch((error) => (document.getElementById('error').innerHTML = error));
}

document.getElementById('form').addEventListener('submit', (e) => {
  e.preventDefault();
  list(document.getElementById('outpoint').value);
});
