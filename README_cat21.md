`cat-21-ord`
=====

Super quick howto to jump into coding.
Generally, this is a great start: https://code.visualstudio.com/docs/languages/rust


1. Install the VS Code extension rust-analyzer: https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer and this if you are on a Mac: https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb

2. Install rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

3. `rustc --version` must work!

4. `cd` into this repo

5. ` cargo build` , once built, the `ord` binary can be found at `./target/release/ord`.

6. `cargo run`

7. You'll need to enable the setting `Debug: Allow Breakpoints Everywhere`, which you can find in the Settings editor (`⌘,`) by searching on 'everywhere`.

8. Generate a new `launch.json` by trying to debug (VS Code should create one for you)

9. Add this to your `launch.json`

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug executable 'ord' INDEX-SATS (Remote RPC Setup)",
            "cargo": {
                "args": [
                    "build",
                    "--bin=ord",
                    "--package=ord"
                ],
                "filter": {
                    "name": "ord",
                    "kind": "bin"
                }
            },
            "args": [
              "--index-sats",
              "--bitcoin-rpc-username=xxx",
              "--bitcoin-rpc-password=yyy",
              "server"
            ],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```
