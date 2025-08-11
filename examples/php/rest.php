<?php
// Minimal REST example without external deps
// Run: php examples/php/rest.php

$url = getenv('FAUXMAIL_HTTP') ?: 'http://127.0.0.1:8025/send';
$payload = json_encode([
  'from' => 'dev@example.test',
  'to' => ['you@example.test'],
  'subject' => 'Hello via REST (PHP)',
  'text' => 'Hi from PHP using REST',
]);
$opts = [
  'http' => [
    'method' => 'POST',
    'header' => "Content-Type: application/json\r\n",
    'content' => $payload,
    'ignore_errors' => true,
  ]
];
$ctx = stream_context_create($opts);
$res = file_get_contents($url, false, $ctx);
echo $res, "\n";

