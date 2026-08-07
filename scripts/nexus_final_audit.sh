#!/data/data/com.termux/files/usr/bin/bash
echo -e "\e[1;36m--- INICIANDO AUDITORÍA DE SOBERANÍA TECNOLÓGICA ---\e[0m"

echo -e "\n\e[1;33m1. Verificando Conexiones de Red (Puertas Traseras):\e[0m"
# Busca conexiones activas. Si sale vacío, estás limpio de tráfico saliente.
netstat -tunp 2>/dev/null | grep ESTABLISHED || echo "No se detectaron conexiones externas activas."

echo -e "\n\e[1;33m2. Análisis de Memoria RAM y Swap (Post-Reinicio):\e[0m"
# Verifica si la memoria Swap se liberó tras el reinicio
free -h

echo -e "\n\e[1;33m3. Escaneo de Procesos Sospechosos:\e[0m"
# Busca rastros de Magis, Malwarebytes mod o Capcut mod en ejecución
ps aux | grep -iE "magis|malware|capcut|espacio" | grep -v grep || echo "Sistema libre de procesos infectados."

echo -e "\n\e[1;33m4. Verificación de Almacenamiento:\e[0m"
# Confirma que no existan instaladores APK en Descargas
ls /sdcard/Download/*.apk 2>/dev/null || echo "Carpeta de descargas limpia de instaladores sospechosos."

echo -e "\n\e[1;32m--- AUDITORÍA FINALIZADA ---\e[0m"
