# PLAN — Instalar Windows 11 (350 GB) y luego retirar Ubuntu
> Generado: 2026-08-11 (Arquitecto / NEXUS)
> Estado: PAUSADO (el Arquitecto se va a dormir). Retomar donde quedó.

## OBJETIVO
- Instalar Windows 11 en 350 GB del NVMe (dual boot con Ubuntu).
- Luego retirar Ubuntu y recuperar la memoria de NEXUS desde Google Drive.
- Sin depender del pendrive de 8 GB (no cabe el ISO de 7,6 GB).

## HARDWARE VERIFICADO
- NVMe KINGSTON SKC3000S1024G de 953,9 GB → p1(1G EFI) + p2(952,8G ext4 Ubuntu)
- **Espacio libre actual: 1.3 MiB** → hay que redimensionar p2 para sacar ~360 GB.
- CPU: Intel Alder Lake, 20 hilos. RAM: 61 GB. GPU: NVIDIA RTX 3070 (8 GB VRAM).
- ufw: deny saliente por defecto; abiertos 53/80/443 + 7171-7180 (Tibia).

## BACKUP DE NEXUS (CRÍTICO — YA HECHO Y VERIFICADO)
En **Google Drive**, carpeta `BACKUP_NEXUS`, vía `rclone` (remoto `gd:`), cuenta crispragma2021@gmail.com:
- `nexus_memoria_dbs.tar.gz`   4.177.411 B  (memoria semántica+episódica+operativa+identidad+ledger)
- `nexus_ocean_lance.tar.gz`   9.397.627 B  (Ocean/LanceDB, vínculos)
- `nexus_brain_sesiones.tar.gz`50.496.426 B (sesiones + identidad)
- `restaurar_nexus.sh`           2.375 B
TODOS CON TAMAÑO EXACTO COINCIDENTE CON LOCAL. Recovery en Windows: descargar de Drive, extraer con 7-Zip.
Código en GitHub: `github.com/crispragma2021/NEXUS_ULTIMATE_CORE` (rama master, sincronizado 2026-08-11 19:01).

## ARCHIVOS/DESCARGA
- `~/Descargas/Win11_25H2_Spanish_Mexico_x64_v2.iso` = 7,7 GB (41.459.210 bytes menos... verificado completo)
  → boot.wim 600M, install.wim 6,8G, `efi/boot/bootx64.efi` presente (booteable UEFI ✓)
- NOTA: la edición así es Windows 11 Home/Pro multiedición (clave desbloquea edición).
- NO se usará Windows modificado (riesgo malware) ni LTSC como bloqueante — re-evaluar, pero ISO oficial ya bajado.

## DECISIÓN TOMADA (Arquitecto)
- 350 GB para Windows. Recuperar memoria rescatada de Ubuntu desde Drive tras eliminar Ubuntu.

## PROCEDIMIENTO (ejecutar con Ubuntu LIVE, no en caliente)
0) PREPARAR: verificar Drive una vez más. Tener ISO de Windows accesible desde el live
   (copiar `Win11_25H2...iso` al pendrive/carpeta de datos ANTES de arrancar el live).
1) Arrancar ISO de Ubuntu (Cruzer) → "Try Ubuntu". Terminal.
2) `sudo lsblk` → confirmar /dev/nvme0n1 con p1(1G EFI) y p2(952G ext4).
3) Redimensionar p2:
   sudo umount /dev/nvme0n1p2
   sudo e2fsck -f /dev/nvme0n1p2
   sudo resize2fs /dev/nvme0n1p2 592G          # deja ~360G libres al final
   sudo sgdisk -p /dev/nvme0n1                 # p2 empieza en sector 2203648
   end = 2203648 + (592G→ sectores) - 1 ≈ 1239572000
   sudo parted /dev/nvme0n1 resizepart 2 1239572000
4) Crear particiones:
   sudo sgdisk -n 3:0:+10G -t 3:ef00 -c 3:"WIN_INSTALLER" /dev/nvme0n1
   sudo sgdisk -n 4:0:+350G -t 4:0700 -c 4:"WINDOWS" /dev/nvme0n1
5) Instalador en partición 3:
   sudo mkfs.vfat -F 32 /dev/nvme0n1p3
   mkdir /mnt/inst; mount p3; mount ISO loop; sudo cp -rL /mnt/isomnt/* /mnt/inst/
6) Añadir entrada GRUB (desde Ubuntu normal o chroot):
   /etc/grub.d/40_custom:
     menuentry "Instalar Windows 11 (disco)" { set root=(hd0,gpt3); chainloader /efi/boot/bootx64.efi; }
   sudo update-grub ; reiniciar → elegir "Instalar Windows 11" → Windows ocupa 350 GB (partición 4).
7) POST-INSTALACIÓN: verificar Windows OK; recuperar memoria desde Drive; retirar Ubuntu
   (partición p2) desde director de discos de Windows; reclamar espacio como Windows puro.

## PENDIENTE PARA MAÑANA / AL REANUDAR
- [ ] Tener el ISO de Windows accesible desde el live (¿copiado al pendrive o a una partición de datos?).
- [ ] Confirmar una vez más que gd:BACKUP_NEXUS tiene los 4 archivos.
- [ ] Ejecutar arriba → montar instalador → GRUB → Windows 350 GB.
- Riesgo alto: redimensionar p2; hacerlo SIEMPRE desde live, nunca en caliente.
