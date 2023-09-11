$INSTALL_DIR="C:\slight"
$OWNER_AND_REPO="deislabs/spiderlightning"
$TAR="slight-windows-x86_64.tar.gz"
$BINARY_NAME="slight.exe"

$LATEST_RELEASE="$(((Invoke-WebRequest "https://api.github.com/repos/$OWNER_AND_REPO/releases").Content | ConvertFrom-Json).tag_name | Select-Object -first 1)"
">>> LATEST RELEASE: $LATEST_RELEASE..."

$URL="https://github.com/$OWNER_AND_REPO/releases/download/$LATEST_RELEASE/$TAR"

New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
">>> BINARY WILL BE STORED AT $INSTALL_DIR."

">>> DONLOADING FROM: $URL..."
Invoke-WebRequest $URL -OutFile "$INSTALL_DIR\$TAR"

tar -xf "$INSTALL_DIR\$TAR" -C $INSTALL_DIR
mv -Force "$INSTALL_DIR\release\$BINARY_NAME" $INSTALL_DIR
">>> EXTRACTED BINARY TAR."

$PATH_CONTENT = [Environment]::GetEnvironmentVariable('path', 'User')
if ($PATH_CONTENT -ne $null)
{
  if ($PATH_CONTENT -split ';'  -NotContains  $INSTALL_DIR)
  {
    [Environment]::SetEnvironmentVariable("Path", [Environment]::GetEnvironmentVariable('Path', 'User') + ";$INSTALL_DIR" , [System.EnvironmentVariableTarget]::User)
    $env:Path += ";$INSTALL_DIR"
  }
}
">>> INSTALLED BINARY."

rm -r -Force "$INSTALL_DIR\$TAR"
rm -r -Force "$INSTALL_DIR\release\"
">>> CLEANED UP."