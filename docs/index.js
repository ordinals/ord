const API_URL = 'http://api.ordinals.com:8000';

const list = (outpoint) => {
  return fetch(`${API_URL}/list/${outpoint}`)
    .then((res) => res.json())
    .then((data) => data)
    .catch((error) => console.log(error));
};

document.getElementById('list-form').addEventListener('submit', (e) => {
  e.preventDefault();
  list(document.getElementById('outpoint').value).then((data) => {
    document.getElementById('list-result').innerHTML = data
      .map((range) => `[${range[0]}, ${range[1]})<br>`)
      .join('');
  });
});
