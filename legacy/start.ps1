# BudgetMaster9000 Launcher (PowerShell)
# Double-click start.bat instead of this file directly.

$ErrorActionPreference = "Stop"
$port = 8769
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host "Starting BudgetMaster9000 local server on port $port..." -ForegroundColor Cyan

try {
    $listener = New-Object System.Net.HttpListener
    $listener.Prefixes.Add("http://localhost:$port/")
    $listener.Start()

    Write-Host "Server running at http://localhost:$port" -ForegroundColor Green
    Write-Host "Close this window to stop the server." -ForegroundColor Yellow

    # Open browser
    Start-Process "http://localhost:$port/index.html"

    while ($listener.IsListening) {
        $context = $listener.GetContext()
        $request = $context.Request
        $response = $context.Response

        $localPath = $request.Url.LocalPath.TrimStart('/')
        if ([string]::IsNullOrWhiteSpace($localPath)) {
            $localPath = "index.html"
        }

        $filePath = Join-Path $scriptDir $localPath

        if (Test-Path $filePath -PathType Leaf) {
            try {
                $bytes = [System.IO.File]::ReadAllBytes($filePath)
                $ext = [System.IO.Path]::GetExtension($filePath).ToLower()
                $response.ContentType = switch ($ext) {
                    ".html" { "text/html; charset=utf-8" }
                    ".js"   { "application/javascript; charset=utf-8" }
                    ".json" { "application/json; charset=utf-8" }
                    ".css"  { "text/css; charset=utf-8" }
                    ".ico"  { "image/x-icon" }
                    default { "application/octet-stream" }
                }
                $response.ContentLength64 = $bytes.Length
                $response.OutputStream.Write($bytes, 0, $bytes.Length)
            } catch {
                $response.StatusCode = 500
                $errorBytes = [System.Text.Encoding]::UTF8.GetBytes("Server error: $($_.Exception.Message)")
                $response.OutputStream.Write($errorBytes, 0, $errorBytes.Length)
            }
        } else {
            $response.StatusCode = 404
            $notFound = [System.Text.Encoding]::UTF8.GetBytes("File not found: $localPath")
            $response.OutputStream.Write($notFound, 0, $notFound.Length)
        }
        $response.OutputStream.Close()
    }
} catch {
    Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
} finally {
    if ($listener) { $listener.Stop() }
}