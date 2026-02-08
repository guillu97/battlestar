# 🚀 Battlestar - Guide de Déploiement Complet

Architecture complète du déploiement:
- **Client (WASM)**: Vercel (CDN global)
- **Serveur (Rust)**: Fly.io (WebSocket + Game Logic)

---

## 📋 Checklist avant déploiement

- [ ] Le jeu fonctionne en local
- [ ] Fly CLI installé (`iwr https://fly.io/install.ps1 -useb | iex`)
- [ ] Vercel CLI installé (`npm install -g vercel`)
- [ ] Connecté à Fly.io (`fly auth login`)
- [ ] Connecté à Vercel (`vercel login`)

---

## 🎯 Déploiement rapide

### Étape 1: Déployer le serveur sur Fly.io

```powershell
cd server
fly launch --no-deploy  # Première fois seulement
fly deploy
fly status  # Noter l'URL: https://battlestar-server.fly.dev
```

### Étape 2: Configurer l'URL WebSocket dans le client

Éditer `client/src/systems/network.rs` (~ligne 56):
```rust
let ws_url = if is_local {
    format!("{}://localhost:3000/ws", ws_protocol)
} else {
    format!("wss://VOTRE-APP.fly.dev/ws")  // ⬅️ Mettre votre URL
};
```

### Étape 3: Builder et déployer le client sur Vercel

```powershell
cd client
trunk build --release  # Génère client/dist/
cd ..
vercel --prod
```

---

## 🧪 Tester localement avant déploiement

### Terminal 1 - Serveur:
```powershell
cd server
cargo run --release
```

### Terminal 2 - Client:
```powershell
cd client
trunk serve --release
```

Ouvrir http://localhost:8080

---

## 📊 Monitoring

### Serveur (Fly.io):
```bash
fly logs              # Logs en temps réel
fly status            # État des machines
fly ssh console       # SSH dans le conteneur
```

### Client (Vercel):
```bash
vercel logs           # Logs
vercel ls             # Liste des déploiements
```

---

## 🔧 Configuration avancée

### Modifier la région Fly.io:
Dans `server/fly.toml`, changer `primary_region`:
- `cdg` - Paris
- `lhr` - London
- `fra` - Frankfurt
- `iad` - Virginia (US East)
- `sjc` - San Jose (US West)

### Augmenter les ressources:
```bash
fly scale vm shared-cpu-1x --memory 1024  # 1GB RAM
```

### Variables d'environnement:
```bash
fly secrets set RUST_LOG=debug
```

---

## 💰 Coûts estimés

**Fly.io** (Serveur):
- Tier gratuit: 3 machines partagées (256MB chacune)
- Avec 512MB: ~2 machines utilisées
- Auto-suspend après inactivité ✅
- **Coût**: Gratuit pour petit trafic

**Vercel** (Client):
- Tier gratuit: 100GB bande passante/mois
- Déploiements illimités
- **Coût**: Gratuit

---

## ⚠️ Troubleshooting

### Le client ne se connecte pas au serveur:
1. Vérifier que l'URL WebSocket est correcte (wss:// pas ws://)
2. Vérifier que le serveur est actif: `fly status`
3. Checker les logs serveur: `fly logs`

### Le serveur s'arrête tout seul:
- Normal! `auto_stop_machines = true` suspend après 5min d'inactivité
- Redémarre automatiquement à la prochaine connexion
- Pour garder actif: `fly scale count 1 --max-per-region 1`

### Build WASM échoue:
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

---

## 🎮 URLs finales

Une fois déployé:

- **Client**: `https://votre-projet.vercel.app`
- **Serveur**: `https://votre-app.fly.dev`
- **WebSocket**: `wss://votre-app.fly.dev/ws`
- **Health check**: `https://votre-app.fly.dev/health`

---

## 📚 Documentation

- [Fly.io Docs](https://fly.io/docs/)
- [Vercel Docs](https://vercel.com/docs)
- [DEPLOY_FLYIO.md](./DEPLOY_FLYIO.md) - Détails serveur
- [DEPLOY_CLIENT.md](./DEPLOY_CLIENT.md) - Détails client
