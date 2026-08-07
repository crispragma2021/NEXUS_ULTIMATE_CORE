#!/bin/bash
echo "🔱 AUDITANDO ESTADO DE VERTEX AI EN EL SILICIO..."
echo "------------------------------------------------"

# 1. Obtener el proyecto activo actual en gcloud
PROJECT_ID=$(gcloud config get-value project 2>/dev/null)

if [ -z "$PROJECT_ID" ]; then
    echo "⚠️  gcloud no tiene un proyecto activo configurado por defecto."
    echo "Intentando extraer de tus variables..."
    # Si sabes tu ID, reemplázalo aquí o usa el de tus credenciales
    PROJECT_ID="project-26e94ab7-4257-4475-ade"
fi

echo "🔍 Proyecto detectado para la auditoría: $PROJECT_ID"
echo ""

# 2. Verificar si el servicio de Vertex AI está habilitado
echo "📡 Consultando servicios activos en Google Cloud API Manager..."
SERVICES_STATUS=$(gcloud services list --project="$PROJECT_ID" --enabled --filter="name:aiplatform.googleapis.com" 2>&1)

if [[ "$SERVICES_STATUS" == *"aiplatform.googleapis.com"* ]]; then
    echo "🟢 ¡ÉXITO! La API de Vertex AI (aiplatform.googleapis.com) ESTÁ HABILITADA en tu proyecto."
else
    echo "🔴 ALERTA: La API de Vertex AI NO está habilitada o no tienes permisos para listarla."
    echo "Trazas del servidor: $SERVICES_STATUS"
    echo ""
    echo "💡 Ejecuta el siguiente comando para intentar forzar su activación:"
    echo "   gcloud services enable aiplatform.googleapis.com --project=$PROJECT_ID"
fi

echo "------------------------------------------------"
# 3. Comprobar si las credenciales predeterminadas de la aplicación (ADC) existen en el host
echo "🔐 Verificando tokens de acceso locales (Application Default Credentials)..."
if [ -f "$HOME/.config/gcloud/application_default_credentials.json" ]; then
    echo "🟢 Archivo ADC localizado en el disco local."
else
    echo "🟡 Archivo ADC faltante. Si Cline usa Vertex de forma nativa, podría fallar aquí."
    echo "💡 Solución para re-autenticar tus credenciales locales:"
    echo "   gcloud auth application-default login"
fi
