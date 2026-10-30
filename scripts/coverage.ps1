param(
    [string]$OutputDirectory = 'target/quality-report',
    [string]$RustToolchain = 'stable-x86_64-pc-windows-msvc',
    [string]$NodeCommand = 'node'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$reportRoot = if ([IO.Path]::IsPathRooted($OutputDirectory)) {
    $OutputDirectory
} else {
    Join-Path $projectRoot $OutputDirectory
}
$previousTarget = $env:CARGO_TARGET_DIR
$previousV8 = $env:NODE_V8_COVERAGE
New-Item -ItemType Directory -Path $reportRoot -Force | Out-Null
Push-Location $projectRoot
try {
    $env:CARGO_TARGET_DIR = Join-Path $projectRoot 'target/coverage-build'
    $env:NODE_V8_COVERAGE = Join-Path $reportRoot 'frontend-v8'
    & cargo "+$RustToolchain" llvm-cov --version 2>&1 | Set-Content (Join-Path $reportRoot 'rust-tool.txt')
    if ($LASTEXITCODE -ne 0) { throw 'Install cargo-llvm-cov before running this script.' }
    & rustc "+$RustToolchain" --version | Add-Content (Join-Path $reportRoot 'rust-tool.txt')
    & $NodeCommand --version | Set-Content (Join-Path $reportRoot 'node-tool.txt')
    if ($LASTEXITCODE -ne 0) { throw 'Node.js 24 or later is required.' }

    & cargo "+$RustToolchain" llvm-cov --locked --html --fail-under-lines 85 `
        --output-dir (Join-Path $reportRoot 'rust') -- --nocapture 2>&1 |
        Tee-Object -FilePath (Join-Path $reportRoot 'rust-tests.log')
    if ($LASTEXITCODE -ne 0) { throw 'Rust tests or the 85 percent line coverage threshold failed.' }
    & cargo "+$RustToolchain" llvm-cov report --json `
        --output-path (Join-Path $reportRoot 'rust/coverage.json')
    if ($LASTEXITCODE -ne 0) { throw 'Could not export Rust coverage JSON.' }
    & cargo "+$RustToolchain" llvm-cov report 2>&1 |
        Tee-Object -FilePath (Join-Path $reportRoot 'rust-summary.txt')
    if ($LASTEXITCODE -ne 0) { throw 'Could not generate Rust coverage summary.' }

    & $NodeCommand --test --test-isolation=none --experimental-test-coverage `
        '--test-coverage-include=**/statics/js/order.js' `
        --test-reporter=spec --test-reporter-destination=stdout `
        --test-reporter=lcov "--test-reporter-destination=$(Join-Path $reportRoot 'frontend.lcov')" `
        tests/frontend.test.cjs 2>&1 | Tee-Object -FilePath (Join-Path $reportRoot 'frontend-tests.log')
    if ($LASTEXITCODE -ne 0) { throw 'Frontend tests failed.' }
    Write-Host "Reports generated in $reportRoot"
} finally {
    $env:CARGO_TARGET_DIR = $previousTarget
    $env:NODE_V8_COVERAGE = $previousV8
    Pop-Location
}
