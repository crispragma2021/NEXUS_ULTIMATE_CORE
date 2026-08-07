// Diagnóstico: captura errores de consola y pageerror de la UI en el puerto dado.
import puppeteer from 'puppeteer'

const url = process.argv[2] || 'http://localhost:5175/'

const browser = await puppeteer.launch({ args: ['--no-sandbox'] })
const page = await browser.newPage()

const errors = []
page.on('console', (msg) => {
  if (msg.type() === 'error') errors.push(`[console.error] ${msg.text()}`)
})
page.on('pageerror', (err) => errors.push(`[pageerror] ${err.message}`))
page.on('requestfailed', (req) =>
  errors.push(`[requestfailed] ${req.url()} → ${req.failure()?.errorText}`)
)

await page.goto(url, { waitUntil: 'networkidle2', timeout: 15000 })

// Esperar un poco a que React monte.
await new Promise((r) => setTimeout(r, 2000))

const bodyText = await page.evaluate(() => document.body.innerText.slice(0, 300))
const rootHtml = await page.evaluate(() => document.getElementById('root').innerHTML.slice(0, 300))
const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor)

console.log('=== BODY TEXT ===')
console.log(JSON.stringify(bodyText))
console.log('=== ROOT HTML ===')
console.log(JSON.stringify(rootHtml))
console.log('=== BODY BACKGROUND ===')
console.log(bg)
console.log('=== ERRORS ===')
console.log(errors.length ? errors.join('\n') : '(ninguno)')

await browser.close()
