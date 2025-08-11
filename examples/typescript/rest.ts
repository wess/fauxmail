// Requires Node 18+ (fetch) or: npm i node-fetch
// Run: ts-node examples/typescript/rest.ts (or compile with tsc)

const url = process.env.FAUXMAIL_HTTP || 'http://127.0.0.1:8025/send';

async function main() {
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      from: 'dev@example.test',
      to: ['you@example.test'],
      subject: 'Hello via REST (TS)',
      text: 'Hi from TypeScript using REST',
    }),
  });
  console.log(await res.text());
}

main().catch((e) => { console.error(e); process.exit(1); });

