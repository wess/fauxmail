// Requires: npm i nodemailer @types/node
// Run: ts-node examples/typescript/send.ts (or compile with tsc)
import nodemailer from 'nodemailer';

async function main() {
  const host = process.env.FAUXMAIL_HOST || '127.0.0.1';
  const port = parseInt(process.env.FAUXMAIL_PORT || '1025', 10);
  const user = process.env.FAUXMAIL_SMTP_USER || undefined;
  const pass = process.env.FAUXMAIL_SMTP_PASS || undefined;

  const transporter = nodemailer.createTransport({
    host,
    port,
    secure: false,
    auth: user && pass ? { user, pass } : undefined,
    tls: { rejectUnauthorized: false },
  });

  const info = await transporter.sendMail({
    from: 'dev@example.test',
    to: 'you@example.test',
    subject: 'Hello from TypeScript',
    text: 'Hi from nodemailer via fauxmail',
  });
  console.log('Sent', info.messageId);
}

main().catch((e) => { console.error(e); process.exit(1); });

