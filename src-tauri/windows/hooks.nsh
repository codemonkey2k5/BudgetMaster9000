; Budget Master 9000 — NSIS installer hooks
; Ensures Start Menu / Desktop shortcuts are removed and recreated on every install
; (including upgrades) so Windows reloads icons from the newly installed EXE.

Var HadDesktopShortcut
Var HadStartMenuShortcut

!macro NSIS_HOOK_PREINSTALL
  StrCpy $HadDesktopShortcut 0
  StrCpy $HadStartMenuShortcut 0

  ; Desktop shortcut
  IfFileExists "$DESKTOP\${PRODUCTNAME}.lnk" 0 bm_pre_no_desk
    StrCpy $HadDesktopShortcut 1
    !insertmacro UnpinShortcut "$DESKTOP\${PRODUCTNAME}.lnk"
    Delete "$DESKTOP\${PRODUCTNAME}.lnk"
  bm_pre_no_desk:

  ; Start Menu shortcut (flat, default Tauri layout)
  IfFileExists "$SMPROGRAMS\${PRODUCTNAME}.lnk" 0 bm_pre_no_sm
    StrCpy $HadStartMenuShortcut 1
    !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    Delete "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  bm_pre_no_sm:

  ; Start Menu shortcut inside product folder (if used)
  IfFileExists "$SMPROGRAMS\${PRODUCTNAME}\${PRODUCTNAME}.lnk" 0 bm_pre_no_smf
    StrCpy $HadStartMenuShortcut 1
    !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}\${PRODUCTNAME}.lnk"
    Delete "$SMPROGRAMS\${PRODUCTNAME}\${PRODUCTNAME}.lnk"
    RMDir "$SMPROGRAMS\${PRODUCTNAME}"
  bm_pre_no_smf:

  ; Start Menu folder chosen by user (MUI start menu page)
  StrCmp $AppStartMenuFolder "" bm_pre_no_smf2
  IfFileExists "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" 0 bm_pre_no_smf2
    StrCpy $HadStartMenuShortcut 1
    !insertmacro UnpinShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
    Delete "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
    RMDir "$SMPROGRAMS\$AppStartMenuFolder"
  bm_pre_no_smf2:
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Always force-refresh shortcuts after files are in place so the shell picks up
  ; the multi-size icon embedded in the new EXE (Tauri skips shortcut creation in /UPDATE).
  ${If} $NoShortcutMode = 0
    ; Start Menu — always recreate (first install + upgrade)
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"

    ; Desktop — restore previous, or create for silent/passive (matches Tauri defaults)
    ${If} $HadDesktopShortcut = 1
    ${OrIf} $PassiveMode = 1
    ${OrIf} ${Silent}
      CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
      !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
    ${EndIf}
  ${EndIf}

  ; Tell Explorer to refresh icons / shortcuts
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
!macroend
