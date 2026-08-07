cat << 'EOF' > nexus_clean_purge.sh
#!/data/data/com.termux/files/usr/bin/bash
echo "Iniciando purga de residuos en Poco X5 Pro..."
# Eliminar todos los APKs descargados que son focos de infección
rm -f /sdcard/Download/*.apk
# Limpiar caché de Termux y archivos temporales
rm -rf $TMPDIR/*
echo "Limpieza de archivos completada."
echo "-----------------------------------"
echo "CONSEJO TÉCNICO: Reinicia el teléfono ahora para vaciar la memoria Swap."
EOF

chmod +x nexus_clean_purge.sh
./nexus_clean_purge.sh
cat << 'EOF' > nexus_final_audit.sh
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
EOF

chmod +x nexus_final_audit.sh
./nexus_final_audit.sh
# 1. Instalar el cliente de SSH
pkg update && pkg upgrade -y
pkg install openssh -y
# 2. Generar tu Llave Maestra (Para entrar sin contraseña después)
ssh-keygen -t rsa -b 4096
# Dale a 'Enter' a todo (no pongas contraseña al archivo).
ssh-copy-id crispragmatico@192.168.1.XX
ssh-copy-id crispragmatico@192.168.1.15
ssh crispragmatico@192.168.0.101
ssh-copy-id crispragmatico@192.168.0.101
.
ssh crispragmatico@192.168.0.101
# Mata cualquier rclone que haya quedado colgado
killall -9 rclone
mkdir -p ~/google_drive
rclone mount gdrive: ~/google_drive --vfs-cache-mode minimal --daemon
ls ~/google_drive
cd ~/FÁBRICA_APPS
python3 -c "from ralph_extractor import iniciar_extraccion_proyectada; iniciar_extraccion_proyectada('/home/crispragmatico/google_drive')"
pkg update && pkg upgrade
pkg install python
exit
pm2 save
exit
pkg install lsof
lsof -i -nP
pkg install lsof -y
lsof -i -nP
find ~/CENTRO_COMANDO/ -mtime -1 -ls
rm -rf $TMPDIR/*
find /sdcard/Download /sdcard/Android/data -name "*.sh" -o -name "*.py" -o -name "*.apk" 2>/dev/null
pkg install net-stat -y
ss -antp
ls $PREFIX/bin | grep -vE '^[a-z]'
pkg install net-tools -y
netstat -ant
ps -ef | grep -v "\[" | grep -v "ps -ef"
cat /etc/resolv.conf
pkg verify
exit
pkg install net-tools
netstat -tupn
pkg install htop
htop
q
exit
curl -X POST http://localhost:5050/api/extract      -H "Content-Type: application/json"      -d '{"target": "google.com"}'
hostname -I
ping -c 4 192.168.1.XX
curl -I http://192.168.1.XX:5050/api/status
ip addr show | grep "inet "
ping -c 4 TU_IP_REAL
hostname -I
ifconfig
pkg install nmap
pkg install nmap -y && nmap -p 5050 192.168.0.0/24
sudo chown -R $USER:$USER ~/CENTRO_COMANDO/postgres_data
n
pkg update && pkg upgrade -y
pkg install htop procps nmap -y
# Limpiar archivos temporales de Termux
apt clean && apt autoremove -y
# Intentar limpiar caché de usuario (si tienes permisos)
rm -rf ~/.cache/*
netstat -tulpn
pkg install android-tools -y
# Listar todas las apps para buscar nombres sospechosos
adb shell pm list packages | grep "search" 
# Sustituye 'nombre.del.paquete' por el sospechoso para eliminarlo del usuario actual
adb shell pm uninstall -k --user 0 nombre.del.paquete
top -m 5 -s cpu
termux-setup-store
termux-setup-storage
cp /sdcard/Download/NEXUS_SNAPSHOT.tar.gz ~
ls ~/storage/download/NEXUS*
ls /sdcard/Download/NEXUS*
cp /sdcard/Download/NEXUS_SURVIVAL_SNAPSHOT.tar.gz ~
ls ~
tar .xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz
rm -rf NEXUS_ULTIMATE_CORE 
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz --no-same-owner --no-same-permissions
rm -rf ~/NEXUS_ULTIMATE_CORE
tar -xzf ~/NEXUS_SURVIVAL_SNAPSHOT.tar.gz --no-same-owner --no-same-permissions
# 1. Ve a tu carpeta personal en Termux
cd ~
# 2. Crea la carpeta del proyecto
mkdir -p NEXUS_ULTIMATE_CORE
# 3. Mueve el snapshot desde las descargas del móvil a Termux
cp /sdcard/Download/NEXUS_SURVIVAL_SNAPSHOT.tar.gz .
# 4. Extrae el sistema de forma limpia
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz -C ~/NEXUS_ULTIMATE_CORE --no-same-owner --no-same-permissions
# 5. Entra y mira tu nuevo reino
cd NEXUS_ULTIMATE_CORE
ls -F
cd ~
rm -rf NEXUS_ULTIMATE_CORE
mkdir NEXUS_ULTIMATE_CORE
cd ~
# Mueve el nuevo snapshot (mucho más rápido)
cp /sdcard/Download/NEXUS_SURVIVAL_SNAPSHOT.tar.gz .
# Extracción limpia y final
mkdir -p NEXUS_ULTIMATE_CORE
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz -C ~/NEXUS_ULTIMATE_CORE --no-same-owner --no-same-permissions
# Entra y verifica (esta vez debería estar todo sin un solo error)
cd NEXUS_ULTIMATE_CORE
ls -F
cd ~
cp ~/storage/downloads/NEXUS_SURVIVAL_SNAPSHOT.tar.gz .
mkdir -p NEXUS_ULTIMATE_CORE
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz -C ~/NEXUS_ULTIMATE_CORE --no-same-owner
termux-setup-storage
rm -rf ~/NEXUS_ULTIMATE_CORE
cd ~
cp ~/storage/downloads/NEXUS_SURVIVAL_SNAPSHOT.tar.gz .
mkdir -p NEXUS_ULTIMATE_CORE
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz -C ~/NEXUS_ULTIMATE_CORE --no-same-owner
cd ~
# Limpia por última vez para evitar fantasmas
rm -rf NEXUS_ULTIMATE_CORE
mkdir -p NEXUS_ULTIMATE_CORE
# Trae el nuevo núcleo (será instantáneo)
cp /sdcard/Download/NEXUS_SURVIVAL_SNAPSHOT.tar.gz .
# Extrae el ADN de NEXUS
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz -C ~/NEXUS_ULTIMATE_CORE --no-same-owner
# Entra y verifica los archivos
cd NEXUS_ULTIMATE_CORE
ls -F
cd ~ && rm -rf NEXUS_ULTIMATE_CORE && mkdir NEXUS_ULTIMATE_CORE
cp /sdcard/Download/NEXUS_SURVIVAL_SNAPSHOT.tar.gz .
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz -C ~/NEXUS_ULTIMATE_CORE --no-same-owner
cd ~
rm -rf NEXUS_ULTIMATE_CORE
mkdir -p NEXUS_ULTIMATE_CORE
# Copiamos el nuevo núcleo limpio
cp /sdcard/Download/NEXUS_SURVIVAL_SNAPSHOT.tar.gz .
# Extracción sin permisos especiales
tar -xzf NEXUS_SURVIVAL_SNAPSHOT.tar.gz -C ~/NEXUS_ULTIMATE_CORE --no-same-owner
# Entramos al bunker
cd NEXUS_ULTIMATE_CORE
ls -F
# Actualiza los repositorios
pkg update && pkg upgrade -y
# Instala el compilador de Rust y herramientas de construcción
pkg install rust clang make pkg-config openssl libsqlite binutils -y
# Si vas a usar funciones de audio/gráficos, instala estas dependencias:
pkg install libxcb libx11 wayland libxkbcommon pango cairo -y
# Añade el repositorio X11
pkg install x11-repo -y
# Reintenta las librerías gráficas
pkg install wayland libxkbcommon cairo pango libxcb -y
# Asegúrate de tener estas también para el orquestador
pkg install openssl libsqlite pkg-config -y
cd ~/NEXUS_ULTIMATE_CORE
cargo build --release -p nexus-orquestador
cd ~/NEXUS_ULTIMATE_CORE
# Elimina la carpeta de configuración que vino de la PC
rm -rf .cargo
# Reintenta la compilación ahora de forma limpia
cargo build --release -p nexus-orquestador
# 1. Asegúrate de que no haya una configuración global que estorbe
rm -rf ~/.cargo/config.toml
# 2. Limpia la variable de entorno que fuerza el uso de sccache
unset RUSTC_WRAPPER
# 3. Y para estar 100% seguros de que no usamos configuraciones heredadas de la PC
# Vamos a lanzar el build forzando el uso del compilador directo:
RUSTC_WRAPPER="" cargo build --release -p nexus-orquestador
termux-wake-lock
export RUST_MIN_STACK=16777216
cargo build --release -p nexus-orquestador -j 1
export RUST_MIN_STACK=67108864
[profile.release]
opt-level = 1           # Reduce la optimización (más fácil para el CPU)
codegen-units = 16      # Permite paralelizar el trabajo (menos carga por unidad)
lto = false             # Desactiva Link Time Optimization temporalmente
panic = "abort"
cargo clean
cargo build --release -p nexus-orquestador -j 1
cp /sdcard/Download/nexus_mobile_ghost.py ~/ && python .nexus_mobile_ghost.py
python
python3 ~/.nexus_mobile_ghost.py
import requests
import os
import json
# EJE DE CONEXIÓN: NÚCLEO MENDOZA
CORE_IP = "192.168.0.103"
URL = f"http://{CORE_IP}:5000/nexus/chat"
def hablar_con_nexus():
if __name__ == "__main__":;     hablar_con_nexus() pkg install imagemagick
cp
sh
find /sdcard -type d -name "*nexus*" -exec du -sh {} + 2>/dev/null | sort -h
exit
cd ~/nexusclaw
cargo build --release --target aarch64-unknown-linux-gnu
trustup target add aarch64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
pkg install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable -y
# Verifica si rustc ya está disponible
which rustc
rustc --version
pkg update
pkg install rust
# 1. Instalar dependencias
pkg install clang binutils make
# 2. Usar el script de instalación oficial (sin rustup, que da problemas)
curl -sSf https://sh.rustup.rs | sh -s -- -y
# 3. Cargar el entorno
source ~/.cargo/env
# 4. Verificar
rustc --version
pkg install wget
wget https://github.com/ollama/ollama/releases/download/v0.6.5/ollama-linux-arm64 -O ollama
chmod +x ollama
./ollama serve &
# Verificar que wget se instaló
which wget
exit
pwd
ls -la
which cargo
cargo --version
ls -la ~/nexusclaw/target/release/nexusclaw 2>/dev/null
ls -la ~/nexusclaw/target/debug/nexusclaw 2>/dev/null
find ~/NEXUS_ULTIMATE_CORE -name "SOUL.md" -o -name "IDENTITY.md" -o -name "MEMORY.md" 2>/dev/null
find ~/nexusclaw -name "SOUL.md" -o -name "IDENTITY.md" -o -name "MEMORY.md" 2>/dev/null
ps aux | grep "20056" | grep -v grep
ps aux | grep "nexus_seed" | grep -v grep
ls -la ~/NEXUS_ULTIMATE_CORE/logs/
tail -5 ~/NEXUS_ULTIMATE_CORE/logs/nexus_orquestador.log 2>/dev/null
sqlite3 ~/nexus_intelligence.db "SELECT COUNT(*) FROM sessions;" 2>/dev/null || echo "No se pudo leer la base de datos"
pkg install sqlite3
pkg install sqlite
rustup update
pkg install binutils build-essential
cd ~/nexusclaw
cargo build --release
No, no son necesariamente iguales. Pueden ser réplicas o versiones distintas del código. Para saberlo, compara su contenido:
```bash
diff -r ~/nexusclaw ~/NEXUS_ULTIMATE_CORE/nexusclaw | head -20
```
· Si no hay salida: son idénticos (uno es copia del otro).
· Si muestra diferencias: son versiones diferentes (p.ej., una más actualizada).
También puedes ver sus fechas de modificación con:
```bash
ls -la ~/nexusclaw/Cargo.toml
ls -la ~/NEXUS_ULTIMATE_CORE/nexusclaw/Cargo.toml
```
El que tenga fecha más reciente probablemente sea la versión más actual. Recomiendo usar el que esté en ~/nexusclaw (raíz) para compilar, porque suele ser el principal.
exit
termux
./install_termux.sh
