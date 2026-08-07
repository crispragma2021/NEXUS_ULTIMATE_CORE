#!/bin/bash
echo "--- CORRIGIENDO PERMISOS Y LANZANDO EN PRIMER PLANO ---"
chmod +x ./proxy_hij

# Aseguramos que el puerto 4444 esté muerto por si acaso
sudo kill -9 $(sudo lsof -t -i:4444) 2>/dev/null

echo "Ejecutando directamente..."
./proxy_hij
