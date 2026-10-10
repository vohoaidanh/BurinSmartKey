@echo off
title Cai dat Burin Smart Key
chcp 65001 >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Setup.ps1"
