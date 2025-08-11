import json, os, urllib.request

url = os.getenv('FAUXMAIL_HTTP', 'http://127.0.0.1:8025/send')
data = {
    'from': 'dev@example.test',
    'to': ['you@example.test'],
    'subject': 'Hello via REST (Python)',
    'text': 'Hi from Python using REST',
}
req = urllib.request.Request(url, data=json.dumps(data).encode('utf-8'), headers={'Content-Type': 'application/json'})
with urllib.request.urlopen(req) as resp:
    print(resp.read().decode('utf-8'))

