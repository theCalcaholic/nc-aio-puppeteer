<?php
declare(strict_types=1);

// increase memory limit to 2GB
// ini_set('memory_limit', '2048M');

// set max execution time to 2h just in case of a very slow internet connection
// ini_set('max_execution_time', '7200');

use \AIO\Data\ConfigurationManager;
use \AIO\Data\Setup;

require __DIR__ . '/../vendor/autoload.php';

$container = \AIO\DependencyInjection::GetContainer();
//$dataConst = $container->get(\AIO\Data\DataConst::class);
//ini_set('session.save_path', $dataConst->GetSessionDirectory());

// Auto logout on browser close
//ini_set('session.cookie_lifetime', '0');

# Keep session for 24h max
//ini_set('session.gc_maxlifetime', '86400');

$configManager = new ConfigurationManager();
$setup = $container->get(Setup::class);
if ($setup->CanBeInstalled()) {
    $setup->Setup();
    $password = null;
    try {

        $config = $configManager->GetConfig();
        // set AIO_TOKEN
        $config['AIO_TOKEN'] = bin2hex(random_bytes(24));
        $configManager->WriteConfig($config);
//        $password = $configManager->GetPassword();
//        echo json_encode(["password" => $password]);
        echo json_encode(["token" => $config['AIO_TOKEN'], "message" => "setup success"]);
        die(200);
    } catch (TypeError) {
        echo json_encode(["error" => "setup failed"]);
        die(500);
    }
}

$token = null;

try {
    $token = $configManager->GetToken();
} catch (TypeError) {
    echo json_encode(["error" => "failed to get AIO token"]);
    die(500);
}
echo json_encode(["token" => $token]);
