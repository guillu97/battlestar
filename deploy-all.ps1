# Full Deployment Script
# Run from project root

param(
    [switch]$ServerOnly,
    [switch]$ClientOnly,
    [string]$ServerUrl
)

$ErrorActionPreference = "Stop"

Write-Host "🎮 Battlestar Deployment Script" -ForegroundColor Magenta
Write-Host "================================" -ForegroundColor Magenta
Write-Host ""

# Check prerequisites
function Test-Command {
    param($Command)
    return (Get-Command $Command -ErrorAction SilentlyContinue) -ne $null
}

if (-not $ClientOnly) {
    if (-not (Test-Command "fly")) {
        Write-Host "❌ Fly CLI not installed" -ForegroundColor Red
        Write-Host "   Install: iwr https://fly.io/install.ps1 -useb | iex" -ForegroundColor Yellow
        exit 1
    }
}

if (-not $ServerOnly) {
    if (-not (Test-Command "trunk")) {
        Write-Host "❌ Trunk not installed" -ForegroundColor Red
        Write-Host "   Install: cargo install trunk" -ForegroundColor Yellow
        exit 1
    }
    
    if (-not (Test-Command "vercel")) {
        Write-Host "❌ Vercel CLI not installed" -ForegroundColor Red
        Write-Host "   Install: npm install -g vercel" -ForegroundColor Yellow
        exit 1
    }
}

# Deploy Server
if (-not $ClientOnly) {
    Write-Host "📡 STEP 1: Deploying Server to Fly.io" -ForegroundColor Cyan
    Write-Host "======================================" -ForegroundColor Cyan
    
    Push-Location server
    
    Write-Host "Building and deploying..." -ForegroundColor Yellow
    fly deploy
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Server deployment failed" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    
    Write-Host "✅ Server deployed!" -ForegroundColor Green
    
    # Get server URL
    $status = fly status | Select-String "Hostname"
    if ($status) {
        $ServerUrl = $status -replace ".*Hostname\s*=\s*", "" -replace "\s.*", ""
        Write-Host "🌐 Server URL: https://$ServerUrl" -ForegroundColor Green
        Write-Host "🔌 WebSocket: wss://$ServerUrl/ws" -ForegroundColor Green
    }
    
    Pop-Location
    Write-Host ""
}

# Deploy Client
if (-not $ServerOnly) {
    Write-Host "🎨 STEP 2: Building and Deploying Client" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    
    if ([string]::IsNullOrEmpty($ServerUrl)) {
        $ServerUrl = Read-Host "Enter your Fly.io server URL (e.g., battlestar-server.fly.dev)"
    }
    
    # Update WebSocket URL in client code
    Write-Host "Updating WebSocket URL to: wss://$ServerUrl/ws" -ForegroundColor Yellow
    
    $networkFile = "client/src/systems/network.rs"
    $content = Get-Content $networkFile -Raw
    
    # Simple regex replacement for the else block
    $pattern = '(?s)(let ws_url = if is_local \{.*?\} else \{).*?(\};)'
    $replacement = "`$1`n                format!(`"wss://$ServerUrl/ws`")`n            `$2"
    $newContent = $content -replace $pattern, $replacement
    
    Set-Content $networkFile $newContent -NoNewline
    
    # Build client
    Push-Location client
    
    Write-Host "Building WASM client..." -ForegroundColor Yellow
    trunk build --release
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Client build failed" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    
    Write-Host "✅ Client built!" -ForegroundColor Green
    Pop-Location
    
    # Deploy to Vercel
    Write-Host "Deploying to Vercel..." -ForegroundColor Yellow
    vercel --prod
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Vercel deployment failed" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "✅ Client deployed!" -ForegroundColor Green
    Write-Host ""
}

Write-Host "🎉 DEPLOYMENT COMPLETE!" -ForegroundColor Green
Write-Host "======================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Open your Vercel URL" -ForegroundColor White
Write-Host "2. Test the game!" -ForegroundColor White
Write-Host "3. Monitor logs:" -ForegroundColor White
Write-Host "   - Server: fly logs" -ForegroundColor Gray
Write-Host "   - Client: vercel logs" -ForegroundColor Gray
Write-Host ""
