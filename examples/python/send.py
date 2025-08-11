import os, smtplib
from email.message import EmailMessage

host = os.getenv('FAUXMAIL_HOST', '127.0.0.1')
port = int(os.getenv('FAUXMAIL_PORT', '1025'))
user = os.getenv('FAUXMAIL_SMTP_USER')
password = os.getenv('FAUXMAIL_SMTP_PASS')

msg = EmailMessage()
msg['From'] = 'dev@example.test'
msg['To'] = 'you@example.test'
msg['Subject'] = 'Hello from Python'
msg.set_content('Hi from smtplib via fauxmail')

with smtplib.SMTP(host, port) as s:
    s.ehlo()
    if user and password:
        s.login(user, password)
    s.send_message(msg)
    print('Sent')

