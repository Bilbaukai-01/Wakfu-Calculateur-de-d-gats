@echo off
chcp 65001 > nul
echo ==========================================
echo    SAUVEGARDE SUR GITHUB (EN COURS...)
echo ==========================================
echo.

:: 1. Préparation des fichiers
git add .

:: 2. Création de la sauvegarde avec la date et l'heure automatique
for /f "tokens=1-3 delims=/ " %%a in ('date /t') do set mydate=%%a-%%b-%%c
for /f "tokens=1-2 delims=: " %%a in ('time /t') do set mytime=%%a-%%b
git commit -m "Mise a jour du %mydate% a %mytime%"

:: 3. Envoi sur GitHub
git push origin main

echo.
echo ==========================================
echo    Sauvegarde terminee avec succes !
echo ==========================================
echo.
timeout /t 3
