import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ARTIFACTS_DIR = path.resolve(__dirname, '../artifacts');

export async function runArchitectureInference(promptStr = '') {
  console.log(`🧠 [AGENT ARCHITECT] Leyendo ui_spec.json e infiriendo arquitectura relacional...`);

  const uiSpecPath = path.join(ARTIFACTS_DIR, 'ui_spec.json');
  const uiSpec = fs.existsSync(uiSpecPath) ? JSON.parse(fs.readFileSync(uiSpecPath, 'utf-8')) : { app_title: 'Sovereign App' };

  const p = (promptStr || '').toLowerCase();
  const includePayments = p.includes('pago') || p.includes('stripe') || p.includes('checkout') || p.includes('suscrip') || p.includes('moneti');

  // Generación de ERD Mermaid
  let mermaidERD = `erDiagram
    users ||--o{ orders : "posee"
    users {
        int id PK
        string name
        string email
        datetime created_at
    }
    orders {
        int id PK
        int user_id FK
        float amount
        string status
        datetime created_at
    }`;

  if (includePayments) {
    mermaidERD += `
    users ||--o{ payments : "realiza"
    payments {
        uuid id PK
        uuid user_id FK
        numeric amount
        string currency
        string stripe_payment_intent
        string status
        datetime created_at
    }`;
  }

  // Generación DDL SQL
  let sqlSchema = `-- DDL Schema generado por Agente Arquitecto NEXUS
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    amount NUMERIC(10,2) NOT NULL,
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
`;

  if (includePayments) {
    sqlSchema += `
CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    amount NUMERIC(10,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    stripe_payment_intent VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'succeeded',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
`;
  }

  // Generación OpenAPI 3.0
  const openApiSpec = {
    openapi: "3.0.0",
    info: {
      title: `${uiSpec.app_title || "Sovereign App"} API`,
      version: "1.0.0",
      description: "API de alta velocidad en Rust (Axum) e integración con Stripe & PostgreSQL"
    },
    paths: {
      "/api/v1/health": {
        get: {
          summary: "Estado del servidor",
          responses: { "200": { description: "Servidor funcionando" } }
        }
      },
      "/api/v1/users": {
        get: { summary: "Obtener lista de usuarios", responses: { "200": { description: "OK" } } },
        post: { summary: "Registrar usuario", responses: { "201": { description: "Creado" } } }
      }
    }
  };

  if (includePayments) {
    openApiSpec.paths["/api/v1/payments/checkout"] = {
      post: {
        summary: "Procesar pago o suscripción con Stripe",
        requestBody: {
          content: {
            "application/json": {
              schema: {
                type: "object",
                properties: {
                  amount: { type: "number", example: 19.99 },
                  currency: { type: "string", example: "USD" },
                  payment_method_id: { type: "string", example: "pm_card_visa" }
                }
              }
            }
          }
        },
        responses: {
          "200": { description: "Pago procesado exitosamente en Stripe y guardado en PostgreSQL" }
        }
      }
    };
  }

  const archSpec = {
    project_name: uiSpec.app_title || "Sovereign App",
    version: "1.0.0",
    database_provider: "postgresql",
    has_stripe_payments: includePayments,
    tables: includePayments ? ["users", "orders", "payments"] : ["users", "orders"],
    schema_file: "artifacts/schema.sql",
    openapi_file: "artifacts/openapi.json",
    mermaid_file: "artifacts/erd.mermaid"
  };

  fs.writeFileSync(path.join(ARTIFACTS_DIR, 'erd.mermaid'), mermaidERD, 'utf-8');
  fs.writeFileSync(path.join(ARTIFACTS_DIR, 'schema.sql'), sqlSchema, 'utf-8');
  fs.writeFileSync(path.join(ARTIFACTS_DIR, 'openapi.json'), JSON.stringify(openApiSpec, null, 2), 'utf-8');
  fs.writeFileSync(path.join(ARTIFACTS_DIR, 'architecture_spec.json'), JSON.stringify(archSpec, null, 2), 'utf-8');

  console.log(`✅ [AGENT ARCHITECT] Arquitectura ${includePayments ? 'con módulo de pagos Stripe' : ''} sincronizada.`);
  return archSpec;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  runArchitectureInference().catch(err => {
    console.error('❌ [AGENT ARCHITECT] Error:', err);
    process.exit(1);
  });
}
