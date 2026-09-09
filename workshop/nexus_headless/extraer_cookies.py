#!/usr/bin/env python3
"""
NEXUS Cookie Extractor v1.0
Extrae cookies de sesión desde perfil persistente de Chrome
para usar con el Headless Engine.
"""
import sqlite3
import os
import json
import shutil
import tempfile
from datetime import datetime

PROFILE_DIR = os.path.expanduser("~/.nexus_profile")
DOMAINS_OF_INTEREST = {
    "facebook.com": "Facebook",
    "instagram.com": "Instagram",
    "linkedin.com": "LinkedIn",
    "telegram.org": "Telegram",
    "tiktok.com": "TikTok",
    "youtube.com": "YouTube",
    "google.com": "Google",
    "twitter.com": "X (Twitter)",
    "x.com": "X (Twitter)",
    "whatsapp.com": "WhatsApp",
    "github.com": "GitHub",
    "reddit.com": "Reddit",
}

def extract_cookies_from_db(db_path):
    """Lee cookies del SQLite de Chrome y filtra por dominios de interés."""
    if not os.path.exists(db_path):
        return {}
    
    # Copiar DB porque Chrome la tiene bloqueada
    tmp = tempfile.NamedTemporaryFile(delete=False)
    try:
        shutil.copy2(db_path, tmp.name)
        tmp.close()
        
        conn = sqlite3.connect(tmp.name)
        cursor = conn.cursor()
        
        cursor.execute("""
            SELECT host_key, name, value, path, expires_utc, is_secure, is_httponly
            FROM cookies
            ORDER BY host_key
        """)
        
        rows = cursor.fetchall()
        conn.close()
        
        cookies_by_domain = {}
        for row in rows:
            host_key, name, value, path, expires_utc, is_secure, is_httponly = row
            
            # Identificar plataforma
            platform = None
            for domain, label in DOMAINS_OF_INTEREST.items():
                if domain in host_key:
                    platform = label
                    break
            
            if not platform:
                continue
            
            if platform not in cookies_by_domain:
                cookies_by_domain[platform] = []
            
            # expires_utc en Chrome es microsegundos desde 1601-01-01
            if expires_utc and expires_utc > 0:
                expires_human = datetime.fromtimestamp(
                    (expires_utc / 1000000) - 11644473600
                ).isoformat() if expires_utc > 0 else "Session"
            else:
                expires_human = "Session"
            
            cookies_by_domain[platform].append({
                "name": name,
                "value": value[:80] + "..." if len(value) > 80 else value,
                "domain": host_key,
                "path": path,
                "secure": bool(is_secure),
                "httponly": bool(is_httponly),
                "expires": expires_human,
            })
        
        os.unlink(tmp.name)
        return cookies_by_domain
    
    except Exception as e:
        if os.path.exists(tmp.name):
            os.unlink(tmp.name)
        return {"error": str(e)}


def extract_cookies_raw(db_path):
    """Extrae cookies completas (valor total) para exportar al engine."""
    if not os.path.exists(db_path):
        return []
    
    tmp = tempfile.NamedTemporaryFile(delete=False)
    try:
        shutil.copy2(db_path, tmp.name)
        tmp.close()
        
        conn = sqlite3.connect(tmp.name)
        cursor = conn.cursor()
        
        cursor.execute("""
            SELECT host_key, name, value, path, expires_utc, is_secure, is_httponly
            FROM cookies
            ORDER BY host_key
        """)
        
        rows = cursor.fetchall()
        conn.close()
        
        cookies_raw = []
        for row in rows:
            host_key, name, value, path, expires_utc, is_secure, is_httponly = row
            
            platform = None
            for domain, label in DOMAINS_OF_INTEREST.items():
                if domain in host_key:
                    platform = label
                    break
            
            if not platform:
                continue
            
            cookies_raw.append({
                "platform": platform,
                "domain": host_key,
                "name": name,
                "value": value,  # Valor completo para usar en requests
                "path": path,
                "secure": bool(is_secure),
                "httponly": bool(is_httponly),
            })
        
        os.unlink(tmp.name)
        return cookies_raw
    
    except Exception as e:
        if os.path.exists(tmp.name):
            os.unlink(tmp.name)
        return []


def main():
    profiles = ["Default", "Profile 1"]
    all_cookies = {}
    all_raw = []
    
    for profile in profiles:
        db_path = os.path.join(PROFILE_DIR, profile, "Cookies")
        if os.path.exists(db_path):
            print(f"\n📂 Perfil: {profile}")
            cookies = extract_cookies_from_db(db_path)
            raw = extract_cookies_raw(db_path)
            
            for platform, platform_cookies in cookies.items():
                if platform not in all_cookies:
                    all_cookies[platform] = []
                all_cookies[platform].extend(platform_cookies)
            
            all_raw.extend(raw)
    
    # Mostrar resumen
    print("\n" + "="*60)
    print("🍪 NEXUS COOKIE EXTRACTOR - RESUMEN")
    print("="*60)
    
    total = 0
    for platform in sorted(all_cookies.keys()):
        count = len(all_cookies[platform])
        total += count
        print(f"  {platform:20s}: {count:3d} cookies")
    
    print(f"\n  {'TOTAL':20s}: {total:3d} cookies")
    
    # Cookies críticas por plataforma (session, token, auth)
    print("\n🔑 COOKIES CRÍTICAS (sesión/autenticación):")
    critical_keywords = ["session", "token", "auth", "sid", "access_token", 
                         "refresh_token", "xs", "c_user", "datr", "sb"]
    
    for cookie in all_raw:
        for kw in critical_keywords:
            if kw in cookie["name"].lower():
                print(f"  [{cookie['platform']:12s}] {cookie['name']}: "
                      f"{cookie['value'][:60]}...")
                break
    
    # Guardar raw para el engine
    out_dir = os.path.join(os.path.dirname(__file__), "cookies")
    os.makedirs(out_dir, exist_ok=True)
    
    raw_path = os.path.join(out_dir, "session_cookies.json")
    with open(raw_path, "w") as f:
        json.dump(all_raw, f, indent=2, ensure_ascii=False)
    
    print(f"\n💾 Cookies exportadas: {raw_path}")
    print(f"   Total: {len(all_raw)} cookies listas para el Headless Engine")


if __name__ == "__main__":
    main()
