; Uninstalling Shellman also takes it off the user's startup list (src-tauri/src/autostart.rs), so no entry
; is left pointing at a deleted exe. An update keeps it: the updater installs with /UPDATE, which doesn't run
; the uninstaller, and the check below makes sure.
!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Shellman"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "Shellman"
  ${EndIf}
!macroend
