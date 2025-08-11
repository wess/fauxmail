<?php
// Requires: composer require phpmailer/phpmailer
// Run: php examples/php/send.php

use PHPMailer\PHPMailer\PHPMailer;
use PHPMailer\PHPMailer\SMTP;

require __DIR__ . '/vendor/autoload.php';

$mail = new PHPMailer(true);
$mail->isSMTP();
$mail->Host = '127.0.0.1';
$mail->Port = 1025;
$user = getenv('FAUXMAIL_SMTP_USER') ?: 'dev';
$pass = getenv('FAUXMAIL_SMTP_PASS') ?: 'secret';
if ($user && $pass) {
  $mail->SMTPAuth = true;
  $mail->Username = $user;
  $mail->Password = $pass;
}
$mail->SMTPSecure = false;
$mail->SMTPAutoTLS = false;
$mail->setFrom('dev@example.test', 'Dev');
$mail->addAddress('you@example.test');
$mail->Subject = 'Hello from PHP';
$mail->Body = "Hi from PHPMailer via fauxmail";
$mail->AltBody = 'Hi';
$mail->send();
echo "Sent\n";

