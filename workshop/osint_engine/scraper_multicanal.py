#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════╗
║  NEXUS OSINT ENGINE — Scraper Multicanal                        ║
║  Búsqueda automatizada en portales Paraguay + redes + DDG Onion ║
║  © 2026 NEXUS — Soberanía Técnica                               ║
╚══════════════════════════════════════════════════════════════════╝
"""
import subprocess
import re
import json
import os
import time
import sys
from pathlib import Path
from datetime import datetime

BASE_DIR = Path("/home/soberano/NEXUS_ULTIMATE_CORE")
OUTPUT_DIR = BASE_DIR / "downloads" / "scraper_results"
REPORT_DIR = BASE_DIR / "reports" / "identities" / "dossier" / "Aldo_Francisco_Coronel_Torres"

# ─── CONFIG TOR ────────────────────────────────────────────────
TOR_PROXY = "socks5://127.0.0.1:9050"
CURL_BASE = ["curl", "-s", "-L", "--max-time", "20", "--proxy", TOR_PROXY,
             "-A", "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0"]


def curl_get(url, via_tor=True):
    """GET request con curl, opcionalmente via Tor."""
    cmd = CURL_BASE.copy() if via_tor else ["curl", "-s", "-L", "--max-time", "20",
                "-A", "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0"]
    cmd.append(url)
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=25)
        return r.stdout
    except Exception as e:
        print(f"     [ERROR] {e}")
        return ""


def curl_post(url, data, via_tor=True):
    """POST request con curl, opcionalmente via Tor."""
    cmd = CURL_BASE.copy() if via_tor else ["curl", "-s", "-L", "--max-time", "20",
                "-A", "Mozilla/5.0"]
    cmd.extend(["--data", data, url])
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=25)
        return r.stdout
    except Exception as e:
        print(f"     [ERROR] {e}")
        return ""


def extract_text(html):
    """Extrae texto visible de HTML."""
    text = re.sub(r'<[^>]+>', ' ', html)
    text = re.sub(r'\s+', ' ', text).strip()
    return text


def extract_links(html, base_url=""):
    """Extrae enlaces de HTML."""
    links = re.findall(r'href=[\'"]?([^\'" >]+)', html)
    return links


# ═══════════════════════════════════════════════════════════════
# 1. BÚSQUEDA EN DUCKDUCKGO ONION
# ═══════════════════════════════════════════════════════════════
DDG_ONION = "duckduckgogg42xjoc72x3sjasowoarfbgcmvfimaftt6twagswzczad.onion"

def search_ddg_onion(query):
    """Búsqueda en DuckDuckGo Onion (requiere POST)."""
    print(f"\n  [🌐] DDG Onion: \"{query[:60]}...\"")
    
    url = f"http://{DDG_ONION}/lite/"
    data = f"q={query.replace(' ', '+')}"
    html = curl_post(url, data)
    
    results = []
    # Extraer resultados del HTML de DDG Lite
    # Formato: <a href="URL" class="result-link">TITLE</a>
    #          <span class="result-snippet">SNIPPET</span>
    
    # Buscar URLs de resultados
    urls = re.findall(r'<a[^>]*href="(https?://[^"]+)"[^>]*class="result-link"[^>]*>([^<]*)</a>', html)
    snippets = re.findall(r'<span[^>]*class="result-snippet"[^>]*>([^<]*)</span>', html)
    
    for i, (url, title) in enumerate(urls):
        snippet = snippets[i] if i < len(snippets) else ""
        results.append({
            'title': extract_text(title),
            'url': url,
            'snippet': extract_text(snippet)
        })
    
    # Si no encontró con el formato esperado, extraer cualquier enlace relevante
    if not results:
        # Extraer todo el texto visible
        text = extract_text(html)
        # Buscar URLs
        urls = re.findall(r'(https?://[^\s<>"\']+)', html)
        for u in urls[:10]:
            results.append({'url': u, 'title': '', 'snippet': ''})
    
    print(f"     └─ {len(results)} resultados")
    return results


def search_google_onion(query):
    """Búsqueda en Google via Tor."""
    print(f"\n  [🌐] Google via Tor: \"{query[:60]}...\"")
    
    url = f"https://www.google.com/search?q={query.replace(' ', '+')}"
    html = curl_get(url)
    
    results = []
    urls = re.findall(r'href="(https?://[^"]*)"[^>]*>', html)
    seen = set()
    
    for u in urls:
        if u not in seen and 'google.com' not in u and len(u) > 15:
            seen.add(u)
            results.append({'url': u[:200], 'title': '', 'snippet': ''})
    
    print(f"     └─ {len(results)} resultados únicos")
    return results


# ═══════════════════════════════════════════════════════════════
# 2. PORTALES JUDICIALES PARAGUAY
# ═══════════════════════════════════════════════════════════════

def search_portal_justicia_paraguay(nombre):
    """Búsqueda en portal de la Corte Suprema de Justicia."""
    print(f"\n  [🏛️] Corte Suprema Paraguay: \"{nombre}\"")
    
    # URL del portal de consulta de causas
    urls_to_try = [
        f"https://www.csj.gov.py/consulta-causas?q={nombre.replace(' ', '+')}",
        f"https://www.csj.gov.py/busqueda?q={nombre.replace(' ', '+')}",
    ]
    
    results = []
    for url in urls_to_try:
        html = curl_get(url)
        text = extract_text(html)
        # Buscar el nombre en el texto
        for part in nombre.split():
            if part.lower() in text.lower():
                # Extraer contexto alrededor
                idx = text.lower().find(part.lower())
                start = max(0, idx - 100)
                end = min(len(text), idx + 200)
                context = text[start:end]
                results.append({
                    'url': url,
                    'context': context[:300],
                    'keyword': part
                })
                break
    
    return results


def search_mspbs_paraguay(nombre):
    """Búsqueda en Ministerio de Salud (registros)."""
    print(f"\n  [🏛️] MSPBS Paraguay: \"{nombre}\"")
    url = f"https://www.mspbs.gov.py/buscar?q={nombre.replace(' ', '+')}"
    html = curl_get(url)
    text = extract_text(html)
    
    if nombre.split()[0].lower() in text.lower():
        return [{'url': url, 'context': text[:500]}]
    return []


def search_set_paraguay(nombre):
    """Búsqueda en SET (Impuestos) - Registro vehicular."""
    print(f"\n  [🏛️] SET Paraguay: \"{nombre}\"")
    # SET no tiene búsqueda pública directa, intentar Google
    return search_google_onion(f"site:set.gov.py {nombre}")


def search_mcit_paraguay(nombre):
    """Búsqueda en Ministerio de Industria y Comercio."""
    print(f"\n  [🏛️] MIC Paraguay: \"{nombre}\"")
    return search_google_onion(f"site:mic.gov.py {nombre}")


# ═══════════════════════════════════════════════════════════════
# 3. BÚSQUEDA DE CHAPA (PATENTE)
# ═══════════════════════════════════════════════════════════════

def search_chapa_online(chapa):
    """Busca una chapa en portales de consulta vehicular."""
    print(f"\n  [🚗] Búsqueda de chapa: \"{chapa}\"")
    
    # Limpiar formato
    chapa_clean = re.sub(r'\s+', '', chapa)
    chapa_encoded = chapa.replace(' ', '+')
    
    sources = [
        # DuckDuckGo Onion
        lambda: search_ddg_onion(f"\"{chapa_clean}\" paraguay automovil vehiculo chapa"),
        # Google via Tor
        lambda: search_google_onion(f"\"{chapa_clean}\" paraguay vehiculo automovil chapa patente"),
        # Buscar en portales paraguayos
        lambda: search_google_onion(f"site:gov.py {chapa_clean} vehiculo"),
        # Búsqueda genérica
        lambda: search_google_onion(f"\"{chapa_clean}\" paraguay"),
    ]
    
    all_results = []
    for source_func in sources:
        try:
            results = source_func()
            all_results.extend(results)
        except Exception as e:
            print(f"     [ERROR] {e}")
    
    return all_results


# ═══════════════════════════════════════════════════════════════
# 4. BÚSQUEDA DE REDES SOCIALES
# ═══════════════════════════════════════════════════════════════

def search_social_media(nombre):
    """Busca perfiles en redes sociales."""
    print(f"\n  [📱] Redes sociales: \"{nombre}\"")
    
    # DuckDuckGo Onion para búsqueda de perfiles
    queries = [
        f"\"{nombre}\" facebook",
        f"\"{nombre}\" linkedin paraguay",
        f"\"{nombre}\" instagram",
        f"\"{nombre}\" twitter",
        f"\"{nombre}\" tiktok",
    ]
    
    all_results = []
    for q in queries:
        try:
            results = search_ddg_onion(q)
            all_results.extend(results)
        except Exception as e:
            print(f"     [ERROR] {q[:30]}: {e}")
    
    return all_results


# ═══════════════════════════════════════════════════════════════
# 5. BÚSQUEDA EN PORTAL GUARANÍ / CAMUS
# ═══════════════════════════════════════════════════════════════

def search_camus_click(nombre):
    """Busca menciones en CAMUS Click."""
    print(f"\n  [📺] CAMUS Click: \"{nombre}\"")
    url = f"https://camusclick.blogspot.com/search?q={nombre.replace(' ', '+')}"
    html = curl_get(url)
    text = extract_text(html)
    
    results = []
    if nombre.split()[0].lower() in text.lower():
        results.append({
            'url': url,
            'context': text[:500],
            'source': 'CAMUS Blog'
        })
    
    # También buscar en Facebook de CAMUS
    fb_results = search_ddg_onion(f"site:facebook.com camusclickpy {nombre}")
    results.extend(fb_results)
    
    return results


def search_portal_guarani(nombre):
    """Busca en Portal Guaraní."""
    print(f"\n  [📖] Portal Guaraní: \"{nombre}\"")
    return search_google_onion(f"site:portalguarani.com {nombre}")


# ═══════════════════════════════════════════════════════════════
# 6. BÚSQUEDA DE VEHÍCULOS / CHAPAS EN PORTALES PÚBLICOS
# ═══════════════════════════════════════════════════════════════

def search_vehicle_registry(chapa_clean):
    """Consulta registros vehiculares públicos."""
    print(f"\n  [🚙] Registro vehicular: \"{chapa_clean}\"")
    
    # Paraguay: no hay API pública, pero intentar scraping
    urls = [
        f"https://www.google.com/search?q=%22{chapa_clean}%22+%22vehiculo%22+paraguay",
        f"https://www.google.com/search?q=%22{chapa_clean}%22+%22automovil%22+paraguay+chapa",
        f"https://www.google.com/search?q=%22{chapa_clean}%22+%22coronel+torres%22+paraguay",
    ]
    
    results = []
    for url in urls:
        html = curl_get(url)
        links = re.findall(r'href="(https?://[^"]*)"[^>]*>', html)
        snippets = re.findall(r'<div[^>]*class="[^"]*BNeawe[^"]*"[^>]*>([^<]*)</div>', html)
        
        for i, link in enumerate(links):
            if chapa_clean[:4] in link or 'coronel' in link.lower() or 'torres' in link.lower():
                snippet = snippets[i] if i < len(snippets) else ""
                results.append({
                    'url': link[:200],
                    'snippet': extract_text(snippet)[:200],
                })
    
    return results


# ═══════════════════════════════════════════════════════════════
# 7. EJECUTOR PRINCIPAL
# ═══════════════════════════════════════════════════════════════

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    
    print("╔══════════════════════════════════════════════════════════╗")
    print("║   NEXUS OSINT ENGINE — Scraper Multicanal                ║")
    print("║   Búsqueda automatizada en 10+ fuentes vía Tor          ║")
    print("╚══════════════════════════════════════════════════════════╝")
    
    # ─── OBJETIVOS ───────────────────────────────────────────
    objetivos = [
        # (nombre, tipo_busqueda)
        ("Aldo Francisco Coronel Torres", "persona"),
        ("Aldo Coronel Torres", "persona"),
        ("Mauricio Cañete", "persona"),
        ("Oscar Mauricio Cañete", "persona"),
        ("CAMUS click Py", "organizacion"),
    ]
    
    # Chapas candidatas (si se generaron del OCR)
    chapa_file = BASE_DIR / "downloads" / "videos" / "chapa_extracts" / "ocr_analysis_report.json"
    chapas_candidatas = []
    if chapa_file.exists():
        try:
            with open(chapa_file) as f:
                data = json.load(f)
                chapas_candidatas = list(data.get('candidates', {}).keys())
        except:
            pass
    
    all_findings = {}
    
    # ─── FASE 1: BÚSQUEDA POR NOMBRE ─────────────────────────
    print(f"\n{'='*60}")
    print("🔍 FASE 1: BÚSQUEDA POR NOMBRE DE PERSONA")
    print(f"{'='*60}")
    
    for nombre, tipo in objetivos:
        print(f"\n{'─'*50}")
        print(f"[🎯] Objetivo: {nombre} ({tipo})")
        
        findings = {
            'nombre': nombre,
            'tipo': tipo,
            'ddg_results': [],
            'google_results': [],
            'judicial_results': [],
            'social_media': [],
            'camus': [],
            'portal_guarani': [],
        }
        
        # DuckDuckGo Onion
        ddg = search_ddg_onion(nombre)
        findings['ddg_results'] = ddg
        
        # Google via Tor
        gg = search_google_onion(nombre)
        findings['google_results'] = gg
        
        # Redes sociales
        sm = search_social_media(nombre)
        findings['social_media'] = sm
        
        # Portal Judicial
        jud = search_portal_justicia_paraguay(nombre)
        findings['judicial_results'] = jud
        
        # Portal Guaraní
        pg = search_portal_guarani(nombre)
        findings['portal_guarani'] = pg
        
        # CAMUS
        if 'camus' in nombre.lower() or 'cañete' in nombre.lower():
            camus = search_camus_click(nombre)
            findings['camus'] = camus
        
        all_findings[nombre] = findings
        
        # Pequeña pausa entre objetivos para no saturar Tor
        time.sleep(2)
    
    # ─── FASE 2: BÚSQUEDA DE CHAPA ──────────────────────────
    print(f"\n{'='*60}")
    print("🚗 FASE 2: BÚSQUEDA DE CHAPA VEHICULAR")
    print(f"{'='*60}")
    
    for chapa in chapas_candidatas:
        results = search_chapa_online(chapa)
        all_findings[f'chapa_{chapa}'] = {
            'chapa': chapa,
            'results': results
        }
        time.sleep(1)
    
    # ─── REPORTE ────────────────────────────────────────────
    print(f"\n{'='*60}")
    print("📊 REPORTE FINAL DE BÚSQUEDAS")
    print(f"{'='*60}\n")
    
    for key, data in all_findings.items():
        if key.startswith('chapa_'):
            chapa = data.get('chapa', key)
            results = data.get('results', [])
            print(f"\n🚗 Chapa: {chapa}")
            if results:
                for r in results[:5]:
                    print(f"  └─ {r.get('url', '')[:120]}")
                    if r.get('snippet'):
                        print(f"     {r['snippet'][:100]}")
            else:
                print(f"  └─ Sin resultados en línea")
        else:
            nombre = data.get('nombre', key)
            total = (len(data.get('ddg_results', [])) + 
                    len(data.get('google_results', [])) +
                    len(data.get('social_media', [])) +
                    len(data.get('judicial_results', [])))
            
            print(f"\n🎯 {nombre}: {total} hallazgos totales")
            
            if data.get('ddg_results'):
                print(f"  🌐 DDG Onion: {len(data['ddg_results'])} resultados")
                for r in data['ddg_results'][:3]:
                    if r.get('title'):
                        print(f"     └─ {r['title'][:80]}")
                    print(f"        {r.get('url', '')[:100]}")
            
            if data.get('google_results'):
                print(f"  🌐 Google Tor: {len(data['google_results'])} URLs")
                for r in data['google_results'][:3]:
                    print(f"     └─ {r.get('url', '')[:100]}")
            
            if data.get('social_media'):
                print(f"  📱 Redes sociales: {len(data['social_media'])} resultados")
                # Agrupar por red
                fb = [r for r in data['social_media'] if 'facebook' in r.get('url', '')]
                li = [r for r in data['social_media'] if 'linkedin' in r.get('url', '')]
                ig = [r for r in data['social_media'] if 'instagram' in r.get('url', '')]
                if fb:
                    print(f"     └─ Facebook: {len(fb)} perfiles")
                    for r in fb[:2]:
                        print(f"        {r.get('url', '')[:100]}")
                if li:
                    print(f"     └─ LinkedIn: {len(li)} perfiles")
                    for r in li[:2]:
                        print(f"        {r.get('url', '')[:100]}")
                if ig:
                    print(f"     └─ Instagram: {len(ig)} perfiles")
            
            if data.get('judicial_results'):
                print(f"  🏛️ Judicial: {len(data['judicial_results'])} menciones")
                for r in data['judicial_results'][:2]:
                    if r.get('context'):
                        print(f"     └─ {r['context'][:120]}")
            
            if data.get('portal_guarani'):
                print(f"  📖 Portal Guaraní: {len(data['portal_guarani'])} resultados")
    
    # ─── GUARDAR RESULTADOS ─────────────────────────────────
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_file = OUTPUT_DIR / f"scraper_multicanal_{timestamp}.json"
    
    with open(output_file, 'w') as f:
        json.dump(all_findings, f, indent=2, default=str)
    
    print(f"\n\n[💾] Resultados guardados: {output_file}")
    
    # Resumen TXT
    summary_file = OUTPUT_DIR / f"scraper_multicanal_{timestamp}.txt"
    with open(summary_file, 'w') as f:
        f.write("=== NEXUS SCRAPER MULTICANAL ===\n\n")
        for key, data in all_findings.items():
            f.write(f"\n[{key}]\n")
            f.write(json.dumps(data, indent=2, default=str)[:2000])
            f.write("\n---\n")
    
    print(f"[💾] Resumen guardado: {summary_file}")
    
    return all_findings


if __name__ == "__main__":
    results = main()
