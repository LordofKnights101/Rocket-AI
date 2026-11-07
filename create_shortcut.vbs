Set ws = CreateObject("WScript.Shell")
Set sc = ws.CreateShortcut(ws.SpecialFolders("Desktop") & "\ROCKET AI Companion.lnk")
sc.TargetPath = "Z:\Coding\Ai Helper (ROCKET)\target\release\rocket-ai.exe"
sc.WorkingDirectory = "Z:\Coding\Ai Helper (ROCKET)"
sc.Description = "ROCKET AI Companion - Your Personal Gaming AI"
sc.Save
MsgBox "Desktop shortcut created successfully!", 0, "ROCKET Setup"