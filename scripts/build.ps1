$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

& tools\tailwindcss.exe -i assets\tailwind.css -o assets\app.css --minify
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

dx bundle --release
exit $LASTEXITCODE
