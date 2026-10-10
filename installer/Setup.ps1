Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$installDir = "$env:LOCALAPPDATA\BurinSmartKey"
$exeSource = Join-Path $PSScriptRoot "BurinSmartKey.exe"
$targetExe = Join-Path $installDir "BurinSmartKey.exe"
$uninstallerBat = Join-Path $installDir "Uninstall.bat"

# Form Setup
$form = New-Object System.Windows.Forms.Form
$form.Text = "Cài đặt Burin Smart Key"
$form.Size = New-Object System.Drawing.Size(460, 360)
$form.StartPosition = "CenterScreen"
$form.FormBorderStyle = "FixedDialog"
$form.MaximizeBox = $false
$form.MinimizeBox = $false
$form.BackColor = [System.Drawing.Color]::White

# Header Panel
$headerPanel = New-Object System.Windows.Forms.Panel
$headerPanel.Size = New-Object System.Drawing.Size(460, 75)
$headerPanel.BackColor = [System.Drawing.Color]::FromArgb(244, 72, 101) # Hồng phấn tươi
$form.Controls.Add($headerPanel)

$titleLabel = New-Object System.Windows.Forms.Label
$titleLabel.Text = "Burin Smart Key"
$titleLabel.Font = New-Object System.Drawing.Font("Segoe UI", 14, [System.Drawing.FontStyle]::Bold)
$titleLabel.ForeColor = [System.Drawing.Color]::White
$titleLabel.Location = New-Object System.Drawing.Point(20, 12)
$titleLabel.AutoSize = $true
$headerPanel.Controls.Add($titleLabel)

$subLabel = New-Object System.Windows.Forms.Label
$subLabel.Text = "Bộ gõ tiếng Việt thông minh thế hệ mới"
$subLabel.Font = New-Object System.Drawing.Font("Segoe UI", 9.5)
$subLabel.ForeColor = [System.Drawing.Color]::FromArgb(254, 226, 226)
$subLabel.Location = New-Object System.Drawing.Point(20, 40)
$subLabel.AutoSize = $true
$headerPanel.Controls.Add($subLabel)

# Body Info
$infoLabel = New-Object System.Windows.Forms.Label
$infoLabel.Text = "Chương trình sẽ được cài đặt vào thư mục người dùng:`n$installDir`n`nTính năng ưu việt:`n• Phím tắt On/Off: Ctrl + Shift | Đổi kiểu gõ: F9`n• Tự động phân biệt & khôi phục tiếng Anh chuẩn xác`n• Icon System Tray hình Chú heo Burin trực quan`n• Chạy nền không hiện màn hình console đen"
$infoLabel.Font = New-Object System.Drawing.Font("Segoe UI", 9)
$infoLabel.Location = New-Object System.Drawing.Point(25, 90)
$infoLabel.Size = New-Object System.Drawing.Size(400, 110)
$form.Controls.Add($infoLabel)

# Checkbox Shortcut
$chkShortcut = New-Object System.Windows.Forms.CheckBox
$chkShortcut.Text = "Tạo biểu tượng ngoài màn hình chính (Desktop Shortcut)"
$chkShortcut.Font = New-Object System.Drawing.Font("Segoe UI", 9)
$chkShortcut.Location = New-Object System.Drawing.Point(25, 210)
$chkShortcut.Size = New-Object System.Drawing.Size(400, 24)
$chkShortcut.Checked = $true
$form.Controls.Add($chkShortcut)

# Checkbox Autorun
$chkAutorun = New-Object System.Windows.Forms.CheckBox
$chkAutorun.Text = "Tự động khởi động cùng Windows khi bật máy"
$chkAutorun.Font = New-Object System.Drawing.Font("Segoe UI", 9)
$chkAutorun.Location = New-Object System.Drawing.Point(25, 235)
$chkAutorun.Size = New-Object System.Drawing.Size(400, 24)
$chkAutorun.Checked = $true
$form.Controls.Add($chkAutorun)

# Install Button
$btnInstall = New-Object System.Windows.Forms.Button
$btnInstall.Text = "Cài đặt ngay"
$btnInstall.Font = New-Object System.Drawing.Font("Segoe UI", 9.5, [System.Drawing.FontStyle]::Bold)
$btnInstall.BackColor = [System.Drawing.Color]::FromArgb(244, 72, 101)
$btnInstall.ForeColor = [System.Drawing.Color]::White
$btnInstall.FlatStyle = [System.Windows.Forms.FlatStyle]::Flat
$btnInstall.Location = New-Object System.Drawing.Point(210, 275)
$btnInstall.Size = New-Object System.Drawing.Size(120, 36)
$form.Controls.Add($btnInstall)

# Cancel Button
$btnCancel = New-Object System.Windows.Forms.Button
$btnCancel.Text = "Hủy"
$btnCancel.Font = New-Object System.Drawing.Font("Segoe UI", 9)
$btnCancel.Location = New-Object System.Drawing.Point(340, 275)
$btnCancel.Size = New-Object System.Drawing.Size(85, 36)
$btnCancel.Add_Click({ $form.Close() })
$form.Controls.Add($btnCancel)

$btnInstall.Add_Click({
    try {
        # 1. Tắt tiến trình cũ nếu đang chạy
        Get-Process "BurinSmartKey", "SmartVietnameseIME", "core_engine" -ErrorAction SilentlyContinue | Stop-Process -Force
        Start-Sleep -Milliseconds 400

        # 2. Tạo thư mục cài đặt
        if (!(Test-Path $installDir)) {
            New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        }

        # 3. Copy File EXE
        Copy-Item -Path $exeSource -Destination $targetExe -Force

        # 4. Tạo file Uninstall.bat
        $batContent = @"
@echo off
chcp 65001 >nul
echo Đang gỡ bỏ Burin Smart Key...
taskkill /F /IM BurinSmartKey.exe /T 2>nul
taskkill /F /IM SmartVietnameseIME.exe /T 2>nul
taskkill /F /IM core_engine.exe /T 2>nul
timeout /t 1 /nobreak >nul
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "BurinSmartKey" /f 2>nul
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "SmartVietnameseIME" /f 2>nul
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\BurinSmartKey" /f 2>nul
del /f /q "%USERPROFILE%\Desktop\Burin Smart Key.lnk" 2>nul
del /f /q "%USERPROFILE%\Desktop\Smart Vietnamese IME.lnk" 2>nul
del /f /q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Burin Smart Key.lnk" 2>nul
del /f /q "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Smart Vietnamese IME.lnk" 2>nul
echo Gỡ bỏ thành công!
pause
"@
        Set-Content -Path $uninstallerBat -Value $batContent -Encoding UTF8

        # 5. Ghi Registry Uninstall vào Windows Control Panel / Settings
        $regPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\BurinSmartKey"
        if (!(Test-Path $regPath)) {
            New-Item -Path $regPath -Force | Out-Null
        }
        Set-ItemProperty -Path $regPath -Name "DisplayName" -Value "Burin Smart Key"
        Set-ItemProperty -Path $regPath -Name "DisplayVersion" -Value "1.0.0"
        Set-ItemProperty -Path $regPath -Name "Publisher" -Value "Burin Software"
        Set-ItemProperty -Path $regPath -Name "DisplayIcon" -Value "$targetExe,0"
        Set-ItemProperty -Path $regPath -Name "UninstallString" -Value "cmd.exe /c `"$uninstallerBat`""
        Set-ItemProperty -Path $regPath -Name "InstallLocation" -Value $installDir
        Set-ItemProperty -Path $regPath -Name "NoModify" -Value 1 -Type DWord
        Set-ItemProperty -Path $regPath -Name "NoRepair" -Value 1 -Type DWord

        # 6. Shortcut Desktop & Start Menu
        $wshShell = New-Object -ComObject WScript.Shell
        if ($chkShortcut.Checked) {
            $desktopShortcut = $wshShell.CreateShortcut("$env:USERPROFILE\Desktop\Burin Smart Key.lnk")
            $desktopShortcut.TargetPath = $targetExe
            $desktopShortcut.WorkingDirectory = $installDir
            $desktopShortcut.Description = "Burin Smart Key - Bộ gõ Tiếng Việt Thông Minh"
            $desktopShortcut.Save()
        }

        $startMenuPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Burin Smart Key.lnk"
        $startMenuShortcut = $wshShell.CreateShortcut($startMenuPath)
        $startMenuShortcut.TargetPath = $targetExe
        $startMenuShortcut.WorkingDirectory = $installDir
        $startMenuShortcut.Save()

        # 7. Khởi động cùng Windows
        if ($chkAutorun.Checked) {
            Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "BurinSmartKey" -Value "`"$targetExe`""
        } else {
            Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "BurinSmartKey" -ErrorAction SilentlyContinue
        }

        # 8. Khởi chạy ứng dụng ngay sau khi cài đặt
        Start-Process -FilePath $targetExe -WorkingDirectory $installDir

        [System.Windows.Forms.MessageBox]::Show("Cài đặt Burin Smart Key thành công!`nỨng dụng đã được khởi chạy với icon chú heo tại khay hệ thống.`n`nPhím tắt:`n• Ctrl + Shift: Bật / Tắt bộ gõ`n• F9: Chuyển đổi TELEX <-> VNI", "Hoàn tất cài đặt", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information)
        $form.Close()
    } catch {
        [System.Windows.Forms.MessageBox]::Show("Có lỗi xảy ra trong quá trình cài đặt: $_", "Lỗi cài đặt", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Error)
    }
})

[void]$form.ShowDialog()
