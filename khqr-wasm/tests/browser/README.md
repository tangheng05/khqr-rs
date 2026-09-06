# Browser test

The Rust unit tests cover the logic behind the bindings, but they run on the
host, where `JsValue` cannot exist. This page runs the real `.wasm` in a real
browser instead.

```sh
wasm-pack build khqr-wasm --target web --out-dir tests/browser/pkg
cd khqr-wasm/tests/browser && python serve.py
```

Then open <http://127.0.0.1:8788>. It rebuilds the published vector, checks the
decode getters, the currency guard, the Khmer path and all three renderers,
and draws the QR it produced.

Headless, for CI or a quick check:

```sh
chrome --headless=new --virtual-time-budget=25000 --dump-dom http://127.0.0.1:8788
```

`serve.py` exists because Windows makes Python's `http.server` label `.js` as
`text/plain`, and browsers refuse module scripts that are not
`text/javascript`.
