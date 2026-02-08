# Battlestar - Déploiement Fly.io

## Prérequis

1. Installer le CLI Fly.io:
```bash
# Windows (PowerShell)
iwr https://fly.io/install.ps1 -useb | iex
```

2. Se connecter à Fly.io:
```bash
fly auth login
```

## Déploiement du serveur

1. Se placer dans le dossier server:
```bash
cd server
```

2. Lancer l'application (première fois):
```bash
fly launch --no-deploy
```

Cela va utiliser le fichier `fly.toml` existant. Confirmer:
- App name: `battlestar-server` (ou votre choix)
- Region: `cdg` (Paris) ou votre préférence
- Ne pas déployer PostgreSQL

3. Déployer:
```bash
fly deploy
```

4. Obtenir l'URL du serveur:
```bash
fly status
```

L'URL sera du type: `https://battlestar-server.fly.dev`

## Mise à jour du client

Une fois le serveur déployé, mettre à jour l'URL WebSocket dans le client:

Dans `client/src/systems/network.rs`, ligne ~56:
```rust
let ws_url = if is_local {
    format!("{}://localhost:3000/ws", ws_protocol)
} else {
    // Remplacer par votre URL Fly.io
    format!("wss://battlestar-server.fly.dev/ws")
};
```

## Commandes utiles

```bash
# Voir les logs
fly logs

# Redémarrer
fly apps restart battlestar-server

# Ouvrir dans le navigateur
fly open

# Surveiller les machines
fly status

# Scaler (augmenter/diminuer)
fly scale count 1

# Détruire l'app
fly apps destroy battlestar-server
```

## Configuration

- **Port interne**: 3000 (Rust Axum)
- **Ports externes**: 80 (HTTP) → redirigé vers 443 (HTTPS)
- **Health check**: `/health` endpoint
- **Auto-scaling**: Démarre/arrête automatiquement selon le trafic
- **Ressources**: 512MB RAM, 1 CPU partagé

## Coût

Le tier gratuit inclut:
- 3 machines partagées
- 256MB RAM par machine (ici on utilise 512MB donc comptera comme ~2 machines)
- Suspension automatique après inactivité

**Note**: Avec `auto_stop_machines = true`, le serveur s'arrête automatiquement après 5 minutes d'inactivité et redémarre à la première connexion (délai ~2-5 secondes).
