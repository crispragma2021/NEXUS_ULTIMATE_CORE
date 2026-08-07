#!/bin/bash

# Este script rotará las claves API que son seguras de rotar automáticamente.
# NO ROTAR claves de Gemini o Vertex AI debido a la estabilidad crítica.

NEXUS_CORE_DIR="/home/soberano/NEXUS_ULTIMATE_CORE"
ENV_FILE="${NEXUS_CORE_DIR}/.env"

log_message() {
  echo "$(date '+%Y-%m-%d %H:%M:%S') - ROTATION SCRIPT: $1" >> "${NEXUS_CORE_DIR}/logs/api_key_rotation.log"
}

# Rotar DeepSeek API Key
if grep -q "DEEPSEEK_API_KEY=" "${ENV_FILE}"; then
  OLD_KEY=$(grep "DEEPSEEK_API_KEY=" "${ENV_FILE}" | cut -d '=' -f2)
  NEW_KEY=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 48 ; echo '') # Generar una nueva clave aleatoria
  sed -i "s/DEEPSEEK_API_KEY=.*/DEEPSEEK_API_KEY=${NEW_KEY}/g" "${ENV_FILE}"
  log_message "DeepSeek API Key rotated. Old: ${OLD_KEY}, New: ${NEW_KEY}"
else
  log_message "DeepSeek API Key not found in .env, skipping rotation."
fi

# Rotar Groq API Key
if grep -q "GROQ_API_KEY=" "${ENV_FILE}"; then
  OLD_KEY=$(grep "GROQ_API_KEY=" "${ENV_FILE}" | cut -d '=' -f2)
  NEW_KEY=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 48 ; echo '') # Generar una nueva clave aleatoria
  sed -i "s/GROQ_API_KEY=.*/GROQ_API_KEY=${NEW_KEY}/g" "${ENV_FILE}"
  log_message "Groq API Key rotated. Old: ${OLD_KEY}, New: ${NEW_KEY}"
else
  log_message "Groq API Key not found in .env, skipping rotation."
fi

# Rotar OpenRouter API Key
if grep -q "OPENROUTER_API_KEY=" "${ENV_FILE}"; then
  OLD_KEY=$(grep "OPENROUTER_API_KEY=" "${ENV_FILE}" | cut -d '=' -f2)
  NEW_KEY=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 48 ; echo '') # Generar una nueva clave aleatoria
  sed -i "s/OPENROUTER_API_KEY=.*/OPENROUTER_API_KEY=${NEW_KEY}/g" "${ENV_FILE}"
  log_message "OpenRouter API Key rotated. Old: ${OLD_KEY}, New: ${NEW_KEY}"
else
  log_message "OpenRouter API Key not found in .env, skipping rotation."
fi

# Rotar Tavily API Key
if grep -q "TAVILY_API_KEY=" "${ENV_FILE}"; then
  OLD_KEY=$(grep "TAVILY_API_KEY=" "${ENV_FILE}" | cut -d '=' -f2)
  NEW_KEY=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 48 ; echo '') # Generar una nueva clave aleatoria
  sed -i "s/TAVILY_API_KEY=.*/TAVILY_API_KEY=${NEW_KEY}/g" "${ENV_FILE}"
  log_message "Tavily API Key rotated. Old: ${OLD_KEY}, New: ${NEW_KEY}"
else
  log_message "Tavily API Key not found in .env, skipping rotation."
fi

log_message "API Key rotation script finished."
