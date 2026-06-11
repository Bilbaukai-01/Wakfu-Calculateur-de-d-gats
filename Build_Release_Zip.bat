@echo off
title Wakfu Calculateur - Build & Packager
echo =======================================================
echo  1. COMPILATION DE L'APPLICATION EN MODE RELEASE...
echo =======================================================
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERREUR] La compilation a echoue !
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo =======================================================
echo  2. CREATION DE L'ARCHIVE ZIP DE MISE A JOUR...
echo =======================================================

:: Définition des chemins exacts avec tes noms de fichiers
set EXE_PATH=target\release\Wakfu_calculateur.exe
set ZIP_PATH=target\release\App_degats-x86_64-pc-windows-msvc.zip

:: Utilisation de PowerShell pour compresser l'exécutable
powershell -Command "Compress-Archive -Path '%EXE_PATH%' -DestinationPath '%ZIP_PATH%' -Force"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERREUR] Impossible de creer le fichier ZIP !
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo =======================================================
echo  OPERATION REUSSIE !
echo =======================================================
echo.
echo Tes fichiers sont prets dans "target/release/" :
echo  [+] L'executable : %EXE_PATH%
echo  [+] L'archive ZIP  : %ZIP_PATH%
echo.
pause
