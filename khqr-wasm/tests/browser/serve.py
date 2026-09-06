"""Static server with correct MIME types.

Windows' registry makes http.server hand back text/plain for .js, and Chrome
refuses module scripts that are not text/javascript.
"""

import http.server
import socketserver

TYPES = {
    ".js": "text/javascript",
    ".mjs": "text/javascript",
    ".wasm": "application/wasm",
    ".html": "text/html",
    ".json": "application/json",
}


class Handler(http.server.SimpleHTTPRequestHandler):
    def guess_type(self, path):
        for suffix, mime in TYPES.items():
            if path.endswith(suffix):
                return mime
        return super().guess_type(path)


socketserver.TCPServer.allow_reuse_address = True
with socketserver.TCPServer(("127.0.0.1", 8788), Handler) as server:
    server.serve_forever()
