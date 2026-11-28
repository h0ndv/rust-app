$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

$tailwind = Join-Path (Get-Location).Path 'tools\tailwindcss.exe'
& $tailwind -i assets\tailwind.css -o assets\app.css --minify
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$watcher = Start-Job -ScriptBlock {
    param($Root)
    Set-Location $Root
    & (Join-Path $Root 'tools\tailwindcss.exe') -i assets\tailwind.css -o assets\app.css --minify --watch
} -ArgumentList (Get-Location).Path

try {
    dx serve --port 1420 --interactive false
    $code = $LASTEXITCODE
}
finally {
    Stop-Job $watcher -ErrorAction SilentlyContinue
    Remove-Job $watcher -Force -ErrorAction SilentlyContinue
}

exit $code
