async function list(outpoint) {
  document.getElementById('result').innerHTML = '';

  try {
    const response = await fetch(
      `http://api.ordinals.com:8000/list/${outpoint}`
    );

    if (!response.ok) {
      const text = await response.text();
      document.getElementById(
        'result'
      ).innerHTML = `${response.statusText}: ${text}`;
      return;
    }

    const ranges = await response.json();

    document.getElementById('result').innerHTML = ranges
      .map((range) => `[${range[0]}, ${range[1]})<br>`)
      .join('');
  } catch (error) {
    document.getElementById('result').innerHTML = `Exception: ${error}`;
  }
}

document.getElementById('form').addEventListener('submit', (e) => {
  e.preventDefault();
  list(document.getElementById('outpoint').value);
});
