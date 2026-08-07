#!/bin/bash
# 🔱 NEXUS: Auditor de Soberanía y Hardware
# Este script verifica la conexión entre el Núcleo y el Santuario.

echo -e "\033[1;32m--- 🔱 REPORTE DE SOBERANÍA OMEGA ---\033[0m"
echo -e "📅 Fecha: $(date)"
echo -e "👤 Usuario: $USER"
echo -e "💻 Host: $(hostname)"

echo -e "\n\033[1;34m[1/3] Verificando Hardware i7-12700F...\033[0m"
lscpu | grep "Model name" || echo "Error al detectar CPU"
free -h | grep "Mem:" | awk '{print "RAM Detectada: " $2}'

echo -e "\n\033[1;34m[2/3] Verificando Manifestación en Documentos...\033[0m"
if [ -f "/home/soberano/Documentos/soberano.txt" ]; then
    echo -e "✅ Archivo 'soberano.txt' encontrado."
    echo -e "📜 Contenido: $(cat /home/soberano/Documentos/soberano.txt)"
else
    echo -e "❌ Alerta: 'soberano.txt' no detectado en Documentos."
fi

echo -e "\n\033[1;34m[3/3] Estado del Perímetro de Red (Port 4444)...\033[0m"
ss -tuln | grep :4444 > /dev/null && echo "✅ Proxy Hijack activo." || echo "⚠️ Proxy Hijack offline."

echo -e "\n\033[1;32m--- FIN DEL REPORTE ---\033[0m"