; Budget Master 9000 — NSIS installer hooks
; Remove stale Start Menu / Desktop shortcuts, then always recreate them after
; install (including upgrades) so Windows reloads icons from the new EXE.
;
; Important: on upgrade the previous uninstaller often deletes the desktop
; shortcut *before* PREINSTALL runs, so "did we have a desktop icon?" is
; unreliable. Desktop is therefore always recreated (same as Start Menu).

!macro NSIS_HOOK_PREINSTALL
  ; Desktop shortcut (current and common alternate names)
  IfFileExists "$DESKTOP\${PRODUCTNAME}.lnk" 0 bm_pre_no_desk
    !insertmacro UnpinShortcut "$DESKTOP\${PRODUCTNAME}.lnk"
    Delete "$DESKTOP\${PRODUCTNAME}.lnk"
  bm_pre_no_desk:

  IfFileExists "$DESKTOP\Budget Master 9000.lnk" 0 bm_pre_no_desk2
    !insertmacro UnpinShortcut "$DESKTOP\Budget Master 9000.lnk"
    Delete "$DESKTOP\Budget Master 9000.lnk"
  bm_pre_no_desk2:

  ; Start Menu shortcut (flat, default Tauri layout)
  IfFileExists "$SMPROGRAMS\${PRODUCTNAME}.lnk" 0 bm_pre_no_sm
    !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    Delete "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  bm_pre_no_sm:

  ; Start Menu shortcut inside product folder (if used)
  IfFileExists "$SMPROGRAMS\${PRODUCTNAME}\${PRODUCTNAME}.lnk" 0 bm_pre_no_smf
    !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}\${PRODUCTNAME}.lnk"
    Delete "$SMPROGRAMS\${PRODUCTNAME}\${PRODUCTNAME}.lnk"
    RMDir "$SMPROGRAMS\${PRODUCTNAME}"
  bm_pre_no_smf:

  ; Start Menu folder chosen by user (MUI start menu page)
  StrCmp $AppStartMenuFolder "" bm_pre_no_smf2
  IfFileExists "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" 0 bm_pre_no_smf2
    !insertmacro UnpinShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
    Delete "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
    RMDir "$SMPROGRAMS\$AppStartMenuFolder"
  bm_pre_no_smf2:
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Always recreate shortcuts after files are in place so the shell picks up
  ; the multi-size icon embedded in the new EXE (Tauri skips shortcut creation
  ; on /UPDATE, and upgrade uninstall may have already removed the desktop icon).
  ${If} $NoShortcutMode = 0
    ; Start Menu — first install + every upgrade
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"

    ; Desktop — always recreate (do not gate on "had shortcut"; upgrade loses that signal)
    CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
    !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
  ${EndIf}

  ; Tell Explorer to refresh icons / shortcuts
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
!macroend
