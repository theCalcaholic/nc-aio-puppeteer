<?php
declare(strict_types=1);

use \AIO\Data\ConfigurationManager;
use \AIO\Data\Setup;

require __DIR__ . '/../vendor/autoload.php';

$container = \AIO\DependencyInjection::GetContainer();

$configManager = new ConfigurationManager();
$setup = $container->get(Setup::class);
if ($setup->CanBeInstalled()) {
    $setup->Setup();
}

try {
    $config = $configManager->GetConfig();
    if (!array_key_exists('AIO_TOKEN', $config)) {
        // set AIO_TOKEN
        $config['AIO_TOKEN'] = bin2hex(random_bytes(24));
        $configManager->WriteConfig($config);
    }
    echo json_encode(["token" => $configManager->GetToken()]);
} catch (TypeError) {
    echo json_encode(["error" => "setup failed"]);
    die(500);
}
