Set shell = CreateObject("WScript.Shell")
repo = CreateObject("Scripting.FileSystemObject").GetParentFolderName(WScript.ScriptFullName)
command = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File """ & repo & "\scripts\start-app.ps1"""
shell.Run command, 0, False
