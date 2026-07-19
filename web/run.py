#!/usr/bin/env python3
"""Launch the BogosortCoin miner and a local web server to visualize it live.

Builds crates/miner in release mode if needed, runs it with --stream-file
pointed at web/state.json, and serves web/ over HTTP so the browser page can
poll that file and animate the permutation as it mines.
"""
import argparse
import http.server
import os
import signal
import socketserver
import subprocess
import sys
import threading
import webbrowser

WEB_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT_DIR = os.path.dirname(WEB_DIR)
STATE_FILE = os.path.join(WEB_DIR, "state.json")
BINARY = os.path.join(ROOT_DIR, "target", "release", "bogosortcoin-miner")


def build_miner():
    print("building bogosortcoin-miner (release)...")
    subprocess.run(
        ["cargo", "build", "--release", "-p", "bogosortcoin-miner"],
        cwd=ROOT_DIR,
        check=True,
    )


def serve(port):
    handler = lambda *a, **kw: http.server.SimpleHTTPRequestHandler(
        *a, directory=WEB_DIR, **kw
    )
    socketserver.TCPServer.allow_reuse_address = True
    httpd = socketserver.TCPServer(("127.0.0.1", port), handler)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    return httpd


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--permutation-size", type=int, default=8)
    parser.add_argument(
        "--target",
        default="0f" * 32,
        help="big-endian hex difficulty target (default: an easy target)",
    )
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--no-browser", action="store_true")
    args = parser.parse_args()

    if not os.path.exists(BINARY):
        build_miner()

    if os.path.exists(STATE_FILE):
        os.remove(STATE_FILE)

    httpd = serve(args.port)
    url = f"http://127.0.0.1:{args.port}/"
    print(f"serving {WEB_DIR} at {url}")
    if not args.no_browser:
        webbrowser.open(url)

    miner = subprocess.Popen(
        [
            BINARY,
            "--permutation-size", str(args.permutation_size),
            "--target", args.target,
            "--stream-file", STATE_FILE,
        ]
    )

    def shutdown(*_):
        miner.terminate()
        httpd.shutdown()
        sys.exit(0)

    signal.signal(signal.SIGINT, shutdown)
    signal.signal(signal.SIGTERM, shutdown)

    miner.wait()
    print("mining finished; server still running, press Ctrl+C to stop.")
    signal.pause()


if __name__ == "__main__":
    main()
