$env:CARGO_TARGET_DIR = "$env:TEMP\mobile_prod_backend_target"
Write-Host "Mengarahkan folder kompilasi ke temporary directory untuk menghindari Windows File Lock (OS Error 32)..."
cargo run
