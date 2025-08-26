# Track Specific Satoshis Feature

This feature allows you to track specific satoshis by their ordinal numbers without having to index all satoshis in the blockchain.

## Usage

### 1. Create a file with satoshi numbers

Create a text file (e.g., `my_sats.txt`) containing the satoshi numbers you want to track:

```
# This is a comment - lines starting with # are ignored
# Each line should contain a satoshi number
0
1
100
1000
10000
50000
100000
```

### 2. Use the --track-sats option

When running ord commands, use the `--track-sats` option to specify your file:

```bash
# Start the server with tracked satoshis
ord --track-sats my_sats.txt server

# Find a specific satoshi
ord --track-sats my_sats.txt find 100

# Find a range of satoshis
ord --track-sats my_sats.txt find 100 200
```

### 3. Benefits

- **Efficient**: Only tracks the satoshis you specify, not all 2.1 quadrillion satoshis
- **Fast**: Much faster indexing and smaller database size
- **Flexible**: Can track any combination of satoshis you're interested in
- **Compatible**: Works with existing ord commands like `find`, `list`, etc.

### 4. How it works

When you use `--track-sats`, ord will:

1. Read the satoshi numbers from your file
2. During indexing, check if any of the satoshis in each transaction output match your list
3. Store the location (UTXO and offset) of matching satoshis in a separate table
4. When you use `find`, it will first check the tracked satoshis table before falling back to the full index (if `--index-sats` is also enabled)

### 5. File format

- One satoshi number per line
- Lines starting with `#` are treated as comments and ignored
- Empty lines are ignored
- Numbers should be valid u64 integers (0 to 2,100,000,000,000,000)

### 6. Example workflow

```bash
# Create a file with satoshis you want to track
echo -e "0\n1\n100\n1000" > my_sats.txt

# Start indexing with tracked satoshis
ord --track-sats my_sats.txt server

# In another terminal, find a tracked satoshi
ord --track-sats my_sats.txt find 100
```

This will output the current location of satoshi 100 in JSON format:

```json
{
  "satpoint": {
    "outpoint": {
      "txid": "abc123...",
      "vout": 0
    },
    "offset": 100
  }
}
``` 