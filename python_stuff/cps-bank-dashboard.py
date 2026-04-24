#!/usr/bin/env python3
import html
import os
import sqlite3
import time
from http.server import BaseHTTPRequestHandler, HTTPServer
from datetime import datetime, timezone

DB = os.environ.get("CPS_DB", "/var/lib/cps-bank/receiver.db")
HOST = os.environ.get("CPS_DASHBOARD_HOST", "0.0.0.0")
PORT = int(os.environ.get("CPS_DASHBOARD_PORT", "8080"))
HUGE_AMOUNT = float(os.environ.get("CPS_HUGE_AMOUNT", "5000"))
STALE_SECONDS = int(os.environ.get("CPS_STALE_SECONDS", "60"))

TX_TYPE = {0: "deposit", 1: "withdrawal", 2: "transfer"}


def q(sql, args=()):
    with sqlite3.connect(DB) as conn:
        conn.row_factory = sqlite3.Row
        return conn.execute(sql, args).fetchall()


def esc(v):
    return html.escape(str(v))


def render():
    now = time.time()
    rows = q("""
        SELECT id, node_id, seq, sent_timestamp_us, account_id, amount, tx_type, received_at
        FROM received_transactions
        ORDER BY id DESC
        LIMIT 100
    """) if os.path.exists(DB) else []

    node_rows = q("""
        SELECT node_id, COUNT(*) AS count, MAX(received_at) AS last_seen
        FROM received_transactions
        GROUP BY node_id
        ORDER BY node_id
    """) if os.path.exists(DB) else []

    alerts = []
    for r in rows[:25]:
        if float(r["amount"]) >= HUGE_AMOUNT:
            alerts.append(f"Huge amount: node={r['node_id']} seq={r['seq']} amount={r['amount']:.2f}")

    for n in node_rows:
        try:
            last = datetime.fromisoformat(str(n["last_seen"]).replace("Z", "+00:00"))
            if last.tzinfo is None:
                last = last.replace(tzinfo=timezone.utc)
            age = now - last.timestamp()
            if age > STALE_SECONDS:
                alerts.append(f"Stale node: node={n['node_id']} last_seen={n['last_seen']} age={int(age)}s")
        except Exception:
            pass

    rows_html = "".join(
        "<tr>"
        f"<td>{esc(r['id'])}</td><td>{esc(r['node_id'])}</td><td>{esc(r['seq'])}</td>"
        f"<td>{esc(r['account_id'])}</td><td>{float(r['amount']):.2f}</td>"
        f"<td>{esc(TX_TYPE.get(r['tx_type'], r['tx_type']))}</td><td>{esc(r['received_at'])}</td>"
        "</tr>" for r in rows
    )

    node_html = "".join(
        f"<tr><td>{esc(n['node_id'])}</td><td>{esc(n['count'])}</td><td>{esc(n['last_seen'])}</td></tr>"
        for n in node_rows
    )

    alerts_html = "".join(f"<li>{esc(a)}</li>" for a in alerts) or "<li>No current alerts.</li>"

    return f"""<!doctype html>
<html><head><meta charset='utf-8'><meta http-equiv='refresh' content='5'>
<title>CPS Bank Monitor</title>
<style>
body{{font-family:system-ui,Arial,sans-serif;margin:2rem;background:#f7f7f7;color:#111}}
.card{{background:white;border:1px solid #ddd;border-radius:12px;padding:1rem;margin-bottom:1rem;box-shadow:0 1px 3px #0001}}
table{{border-collapse:collapse;width:100%;background:white}}td,th{{border-bottom:1px solid #eee;padding:.45rem;text-align:left}}
.alerts li{{margin:.25rem 0}} code{{background:#eee;padding:.15rem .3rem;border-radius:4px}}
</style></head><body>
<h1>CPS Distributed Bank Transaction Monitor</h1>
<div class='card'><b>DB:</b> <code>{esc(DB)}</code> · <b>Refresh:</b> 5s · <b>Huge amount threshold:</b> {HUGE_AMOUNT}</div>
<div class='card'><h2>Alerts</h2><ul class='alerts'>{alerts_html}</ul></div>
<div class='card'><h2>Nodes</h2><table><tr><th>Node</th><th>Transactions</th><th>Last seen UTC</th></tr>{node_html}</table></div>
<div class='card'><h2>Last 100 transactions</h2><table><tr><th>ID</th><th>Node</th><th>Seq</th><th>Account</th><th>Amount</th><th>Type</th><th>Received UTC</th></tr>{rows_html}</table></div>
</body></html>"""


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path not in ("/", "/index.html"):
            self.send_error(404)
            return
        body = render().encode()
        self.send_response(200)
        self.send_header("content-type", "text/html; charset=utf-8")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt, *args):
        return


if __name__ == "__main__":
    HTTPServer((HOST, PORT), Handler).serve_forever()
