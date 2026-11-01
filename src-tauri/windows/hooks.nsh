; Arquivos do Trasno: limpeza ao desinstalar.
; O menú do clic dereito do Explorador vive no rexistro do usuario (HKCU); quítase ao desinstalar,
; pero non cando só se está actualizando a app (o instalador novo volve poñer o mesmo programa).
; A caixa «Borrar os datos da aplicación» do desinstalador decide se tamén se borran os datos propios.

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode != 1
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.png\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.jpg\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.jpeg\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.gif\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.bmp\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.ico\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.tiff\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.tif\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.webp\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.tga\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.qoi\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.mp3\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.wav\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.flac\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.ogg\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.opus\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.m4a\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.aac\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.wma\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.aiff\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.mp4\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.mkv\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.mov\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.avi\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.webm\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.flv\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.wmv\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.mpg\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.mpeg\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.txt\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.md\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.csv\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.json\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.xml\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.log\shell\ArquivosDoTrasno"
    DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.pdf\shell\ArquivosDoTrasno"
    ${If} $DeleteAppDataCheckboxState == 1
      ; rexistro de procesos e historial
      RMDir /r "$LOCALAPPDATA\ArquivosDoTrasno"
    ${EndIf}
  ${EndIf}
!macroend
