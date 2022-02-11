<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hello Rustling</title>
</head>
<body>
    <h1>Hello Rustling</h1>
    <p><?php
error_reporting(E_ALL);
file_put_contents('php://stderr', 'my message');
$test = explode("=",$argv[1]);
$number = explode("=",$argv[2]);
echo $test[0] . " = " . $test[1];
echo $number[0] . " = " . $number[1];
    ?></p>
</body>
</html>