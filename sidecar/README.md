# ParseDock Node sidecar (dev-only)

This package is for **local development and debugging** of the JSON stdin/stdout protocol.

**Production builds** use the Rust binary built by `scripts/build-sidecar.sh` (`parsedock-sidecar`), not this Node script.

```bash
cd sidecar && npm install
echo '{"files":["/path/to/doc.pdf"],"outputDir":"/tmp/out","format":"md","ocrEnabled":false,"workers":1}' | node index.js
```