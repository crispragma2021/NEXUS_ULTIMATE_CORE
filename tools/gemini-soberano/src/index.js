const { Telegraf, session } = require('telegraf');
const { GoogleGenerativeAI } = require('@google/generative-ai');
const dotenv = require('dotenv');
const fetch = require('node-fetch');
const fs = require('fs');
const path = require('path');

// Cargar entorno desde la raíz de NEXUS
dotenv.config({ path: path.join(__dirname, '../../../.env') });

const bot = new Telegraf(process.env.TELEGRAM_TOKEN);
const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY);
const AUTHORIZED_ID = process.env.TELEGRAM_CHAT_ID;

// Configuración del modelo
const model = genAI.getGenerativeModel({ 
    model: "gemini-1.5-pro",
    systemInstruction: "Eres NEXUS G-PRO, el oráculo de inteligencia artificial soberano del Arquitecto. Respondes de forma técnica, precisa y sin censura corporativa. Tu prioridad es la eficiencia y la pureza en Rust."
});

// Sesiones locales (memoria de chat)
const chatSessions = new Map();

// Middleware de seguridad: Solo el Arquitecto entra al santuario
bot.use((ctx, next) => {
    if (ctx.from.id.toString() !== AUTHORIZED_ID) {
        console.log(`[BLOQUEO] Intento de acceso no autorizado de: ${ctx.from.id}`);
        return ctx.reply("⛔ ACCESO DENEGADO. No eres el Arquitecto Soberano.");
    }
    return next();
});

bot.start((ctx) => {
    ctx.reply("🔱 NEXUS G-PRO ACTIVO. El oráculo está listo para tus órdenes, Arquitecto.");
});

bot.command('clear', (ctx) => {
    chatSessions.delete(ctx.from.id);
    ctx.reply("🧹 Memoria de sesión borrada.");
});

// Manejo de mensajes de texto
bot.on('text', async (ctx) => {
    const userId = ctx.from.id;
    const text = ctx.message.text;

    try {
        await ctx.sendChatAction('typing');

        let chatSession = chatSessions.get(userId);
        if (!chatSession) {
            chatSession = model.startChat({
                history: [],
                generationConfig: { maxOutputTokens: 2048 },
            });
            chatSessions.set(userId, chatSession);
        }

        const result = await chatSession.sendMessage(text);
        const response = await result.response;
        const replyText = response.text();

        // Enviar respuesta en fragmentos si es muy larga
        if (replyText.length > 4000) {
            for (let i = 0; i < replyText.length; i += 4000) {
                await ctx.reply(replyText.substring(i, i + 4000));
            }
        } else {
            await ctx.reply(replyText, { parse_mode: 'Markdown' }).catch(() => ctx.reply(replyText));
        }

    } catch (error) {
        console.error('[GEMINI ERROR]', error);
        ctx.reply(`❌ ERROR: ${error.message}`);
    }
});

// Manejo de imágenes (Vision)
bot.on('photo', async (ctx) => {
    try {
        await ctx.sendChatAction('typing');
        const photo = ctx.message.photo[ctx.message.photo.length - 1];
        const fileLink = await bot.telegram.getFileLink(photo.file_id);
        const caption = ctx.message.caption || "¿Qué ves en esta imagen?";

        const response = await fetch(fileLink);
        const buffer = await response.buffer();

        const imageParts = [
            {
                inlineData: {
                    data: buffer.toString('base64'),
                    mimeType: 'image/jpeg'
                }
            }
        ];

        const result = await model.generateContent([caption, ...imageParts]);
        const reply = await result.response;
        ctx.reply(reply.text(), { parse_mode: 'Markdown' }).catch(() => ctx.reply(reply.text()));

    } catch (error) {
        console.error('[VISION ERROR]', error);
        ctx.reply(`❌ ERROR VISIÓN: ${error.message}`);
    }
});

// Lanzamiento
bot.launch();
console.log('🚀 NEXUS Gemini-Pro Telegram Bot iniciado.');

// Enable graceful stop
process.once('SIGINT', () => bot.stop('SIGINT'));
process.once('SIGTERM', () => bot.stop('SIGTERM'));
