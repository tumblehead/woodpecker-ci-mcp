# Upload a file as an asset of the Gitea release for $env:CI_COMMIT_TAG,
# creating the release if needed. Any failed Gitea request prints the HTTP
# status and response body and fails the step.
#
# Usage: upload-release-asset.ps1 -File <path>
# Env:   GITEA_TOKEN, CI_FORGE_URL, CI_REPO, CI_COMMIT_TAG
# Works on Windows PowerShell 5.1 and PowerShell 7.
param([Parameter(Mandatory = $true)][string]$File)

$ErrorActionPreference = 'Stop'

$name = Split-Path $File -Leaf
$tag = $env:CI_COMMIT_TAG
$api = "$env:CI_FORGE_URL/api/v1/repos/$env:CI_REPO/releases"
$headers = @{ Authorization = "token $env:GITEA_TOKEN" }

function Get-Status($err) {
    if ($err.Exception.Response) { [int]$err.Exception.Response.StatusCode } else { 0 }
}

function Fail($what, $err) {
    $detail = if ($err.ErrorDetails.Message) { $err.ErrorDetails.Message } else { $err.Exception.Message }
    [Console]::Error.WriteLine("error: $what failed with HTTP $(Get-Status $err): $detail")
    exit 1
}

# The builds run in parallel, so another platform may have created it already (409)
try {
    $body = @{ tag_name = $tag; name = $tag } | ConvertTo-Json
    Invoke-RestMethod -Method Post -Uri $api -Headers $headers -ContentType 'application/json' -Body $body | Out-Null
    Write-Host "Created release $tag"
} catch {
    if ((Get-Status $_) -ne 409) { Fail "creating release $tag" $_ }
    Write-Host "Release $tag already exists"
}

try {
    $release = Invoke-RestMethod -Uri "$api/tags/$tag" -Headers $headers
} catch {
    Fail "looking up release $tag" $_
}

try {
    Invoke-RestMethod -Method Post -Uri "$api/$($release.id)/assets?name=$name" -Headers $headers `
        -InFile $File -ContentType 'application/octet-stream' | Out-Null
} catch {
    Fail "uploading $name" $_
}
Write-Host "Uploaded $name to release $tag"
