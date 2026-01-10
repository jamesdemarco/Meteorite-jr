# Meteorite-jr




Project requires Microsoft Visual Studio Build Tools with Windows SDK (11.0)


1.) From powershell run:

winget install --id Microsoft.VisualStudio.2022.BuildTools -e --accept-package-agreements --accept-source-agreements


2.) when visual studio installer window opes: 

    - select visual studio build tools
    - expand to see all options
    - find and select windows sdk 11.0


3.) From vscode terminal project root

cargo check








 CAn run in both Mock and Real scenarios


 Real: Cargo run


 Mock: cargo run --no-default-features --features mock