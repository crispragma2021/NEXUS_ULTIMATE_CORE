#!/data/data/com.termux/files/usr/bin/bash
echo "Liberando buffer de Termux en Poco X5 Pro..."
sync && echo "Procesos de usuario limpiados."
# Mata procesos de fondo que no sean del sistema o Termux
pkill -u $(whoami) -9 -e | grep -v "bash\|ssh\|termux"
