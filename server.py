#!/usr/bin/env python3
import http.server
import socketserver
import os
import sys
import json
from urllib.parse import urlparse, parse_qs

PORT = 8080

def resolve_use_desc(atom, flag):
    # 1. Search local ebuild-specific USE flag descriptions
    paths_local = [
        "/var/db/repos/gentoo/profiles/use.local.desc",
        "/usr/portage/profiles/use.local.desc"
    ]
    for path in paths_local:
        if os.path.exists(path):
            try:
                prefix = f"{atom}:{flag}"
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    for line in f:
                        line = line.strip()
                        if line.startswith(prefix):
                            parts = line.split(" - ", 1)
                            if len(parts) == 2:
                                return parts[1].strip()
            except:
                pass

    # 2. Search global USE flag descriptions
    paths_global = [
        "/var/db/repos/gentoo/profiles/use.desc",
        "/usr/portage/profiles/use.desc"
    ]
    for path in paths_global:
        if os.path.exists(path):
            try:
                with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                    for line in f:
                        line = line.strip()
                        if not line or line.startswith('#'):
                            continue
                        parts = line.split(" - ", 1)
                        if len(parts) == 2 and parts[0].strip() == flag:
                            return parts[1].strip()
            except:
                pass

    return f"Enable {flag} support"

def extract_ebuild(pkg_path, pkg_name, atom=""):
    try:
        ebuilds = [f for f in os.listdir(pkg_path) if f.endswith(".ebuild")]
        if not ebuilds:
            return None
        ebuilds.sort()
        latest = ebuilds[-1]
        
        # Version extraction
        version = latest.replace(f"{pkg_name}-", "").replace(".ebuild", "")
        
        homepage = ""
        description = ""
        license = ""
        use_flags = []
        dependencies = []
        
        with open(os.path.join(pkg_path, latest), 'r', encoding='utf-8', errors='ignore') as f:
            in_rdepend = False
            rdepend_raw = ""
            
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                if line.startswith("HOMEPAGE="):
                    homepage = line.split("=", 1)[1].strip('"\'` ')
                elif line.startswith("DESCRIPTION="):
                    description = line.split("=", 1)[1].strip('"\'` ')
                elif line.startswith("LICENSE="):
                    license = line.split("=", 1)[1].strip('"\'` ')
                elif line.startswith("IUSE="):
                    iuse_val = line.split("=", 1)[1].strip('"\'` ')
                    for flag in iuse_val.split():
                        default = False
                        if flag.startswith("+"):
                            flag = flag[1:]
                            default = True
                        # Skip compiler target subflags to keep output clean
                        if "_" in flag and not flag.startswith("python_targets"):
                            continue
                        use_flags.append({
                            "name": flag,
                            "description": resolve_use_desc(atom, flag),
                            "default": default
                        })
                
                # RDEPEND dependencies parsing (handling multi-line blocks)
                if line.startswith("RDEPEND="):
                    in_rdepend = True
                    val = line.split("=", 1)[1]
                    rdepend_raw += val
                    if val.count('"') % 2 == 0 and '"' in val:
                        in_rdepend = False
                elif in_rdepend:
                    rdepend_raw += " " + line
                    if '"' in line:
                        in_rdepend = False
                        
        # Clean RDEPEND package list
        clean_rdep = rdepend_raw.strip('"\'`() ')
        for token in clean_rdep.split():
            if token.startswith(">") or token.startswith("<") or token.startswith("=") or token in ["||", "!"]:
                continue
            token = token.lstrip("><=!")
            if "/" in token:
                parts = token.split("-")
                base_parts = []
                for p in parts:
                    if p and (p[0].isdigit() or p.startswith("r")):
                        break
                    base_parts.append(p)
                atom_path = "-".join(base_parts)
                clean_atom = atom_path.split("[")[0].split(":")[0]
                if "/" in clean_atom and clean_atom not in dependencies:
                    dependencies.append(clean_atom)
                    
        return {
            "version": version,
            "homepage": homepage,
            "description": description,
            "license": license,
            "use_flags": use_flags,
            "dependencies": dependencies[:5]
        }
    except Exception as e:
        return None

def scan_repos(query):
    query = query.lower().strip()
    results = []
    repos_dir = "/var/db/repos"
    if not os.path.exists(repos_dir):
        return results
        
    try:
        # Loop through repos (gentoo, guru, etc.)
        for repo in os.listdir(repos_dir):
            repo_path = os.path.join(repos_dir, repo)
            if not os.path.isdir(repo_path) or repo.startswith('.'):
                continue
                
            # Loop through categories (app-editors, sys-apps, etc.)
            for cat in os.listdir(repo_path):
                cat_path = os.path.join(repo_path, cat)
                if not os.path.isdir(cat_path) or cat in ["profiles", "metadata", "eclass", "licenses"]:
                    continue
                    
                # Loop through packages
                for pkg in os.listdir(cat_path):
                    pkg_path = os.path.join(cat_path, pkg)
                    if not os.path.isdir(pkg_path):
                        continue
                        
                    atom = f"{cat}/{pkg}"
                    if query in atom.lower() or query in pkg.lower():
                        info = extract_ebuild(pkg_path, pkg, atom)
                        if info:
                            results.append({
                                "name": pkg,
                                "atom": atom,
                                "version": info["version"],
                                "overlay": repo,
                                "homepage": info["homepage"],
                                "description": info["description"],
                                "license": info["license"],
                                "use_flags": info["use_flags"],
                                "dependencies": info["dependencies"],
                                "masked": False
                            })
    except Exception as e:
        print(f"Error scanning repos: {e}", file=sys.stderr)
        
    # Sort results alphabetically
    results.sort(key=lambda x: x["atom"])
    return results

class EzMergeHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        parsed_url = urlparse(self.path)
        
        # Dynamic search API endpoint
        if parsed_url.path == "/api/search":
            query = parse_qs(parsed_url.query).get('q', [''])[0].strip()
            
            # Search live repositories
            results = []
            if query:
                results = scan_repos(query)
                
            # Fallback to local db.json if repo crawling is empty/not on Gentoo
            if not results:
                try:
                    db_path = os.path.join(os.getcwd(), "ezmerge-api", "db.json")
                    with open(db_path, 'r', encoding='utf-8') as f:
                        db = json.load(f)
                    if query:
                        results = [
                            p for p in db["packages"]
                            if query in p["name"].lower() or 
                               query in p["atom"].lower() or 
                               query in p["description"].lower()
                        ]
                    else:
                        results = db["packages"]
                except Exception as e:
                    print(f"Fallback database error: {e}", file=sys.stderr)
                    
            # Send JSON response
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.send_header('Cache-Control', 'no-store, no-cache, must-revalidate')
            self.end_headers()
            self.wfile.write(json.dumps(results).encode('utf-8'))
            return
            
        super().do_GET()

    def translate_path(self, path):
        # Redirect API static queries to central db.json
        if path.startswith("/ezmerge-api/db.json") or path.startswith("/api/db.json") or path == "/db.json":
            return os.path.join(os.getcwd(), "ezmerge-api", "db.json")
            
        # Clean paths for static routing
        clean_path = path.split('?')[0].split('#')[0]
        relative_path = clean_path.lstrip('/')
        
        web_dir = os.path.join(os.getcwd(), "ezmerge-web")
        target_path = os.path.abspath(os.path.join(web_dir, relative_path))
        
        # Directory traversal prevention
        if not target_path.startswith(os.path.abspath(web_dir)):
            return os.path.join(web_dir, "index.html")
            
        if os.path.isdir(target_path):
            return os.path.join(target_path, "index.html")
            
        if not os.path.exists(target_path):
            return os.path.join(web_dir, "index.html")
            
        return target_path

    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        super().end_headers()

if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)
    
    socketserver.TCPServer.allow_reuse_address = True
    print(f"==================================================")
    print(f"🚀 ezMerge Web Service & Live System API Portal")
    print(f"   URL: http://localhost:8080")
    print(f"==================================================")
    
    try:
        with socketserver.TCPServer(("", PORT), EzMergeHandler) as httpd:
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down ezMerge server.")
        sys.exit(0)
    except Exception as e:
        print(f"Error starting server: {e}", file=sys.stderr)
        sys.exit(1)
