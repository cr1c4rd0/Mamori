# MAMORI — Script de ejecución rápida para PowerShell
Stop-Process -Name "mamori-client" -Force -ErrorAction SilentlyContinue
$env:Path = "$HOME\.cargo\bin;" + $env:Path
cargo run -p mamori-client
