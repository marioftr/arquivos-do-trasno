# Actualiza, compila Trasno como .exe real e ábreo. Uso: .\scripts\abrir.ps1
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)
git pull
npm install
npm run tauri build -- --no-bundle
$exe = Join-Path (Get-Location) "target\release\trasno.exe"
if (-not (Test-Path $exe)) { throw "Non se xerou $exe" }
Start-Process $exe
